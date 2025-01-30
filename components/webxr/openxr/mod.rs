use crate::gl_utils::GlClearer;
use crate::SurfmanGL;

use euclid::Box2D;
use euclid::Point2D;
use euclid::Rect;
use euclid::RigidTransform3D;
use euclid::Rotation3D;
use euclid::Size2D;
use euclid::Transform3D;
use euclid::Vector3D;
use glow::PixelUnpackData;
use glow::{self as gl, HasContext};
use interaction_profiles::{get_profiles_from_path, get_supported_interaction_profiles};
use log::{error, warn};
use openxr::sys::CompositionLayerPassthroughFB;
use openxr::{
    self, ActionSet, ActiveActionSet, ApplicationInfo, CompositionLayerBase, CompositionLayerFlags,
    CompositionLayerProjection, Entry, EnvironmentBlendMode, ExtensionSet, Extent2Di, FormFactor,
    Fovf, FrameState, FrameStream, FrameWaiter, Graphics, Instance, Passthrough,
    PassthroughFlagsFB, PassthroughLayer, PassthroughLayerPurposeFB, Posef, Quaternionf,
    ReferenceSpaceType, SecondaryEndInfo, Session, Space, Swapchain, SwapchainCreateFlags,
    SwapchainCreateInfo, SwapchainUsageFlags, SystemId, Vector3f, Version, ViewConfigurationType,
};
use std::collections::HashMap;
use std::mem;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use surfman::Context as SurfmanContext;
use surfman::Device as SurfmanDevice;
use surfman::Error as SurfmanError;
use surfman::SurfaceTexture;
use webxr_api;
use webxr_api::util::{self, ClipPlanes};
use webxr_api::BaseSpace;
use webxr_api::Capture;
use webxr_api::ContextId;
use webxr_api::DeviceAPI;
use webxr_api::DiscoveryAPI;
use webxr_api::Display;
use webxr_api::Error;
use webxr_api::Event;
use webxr_api::EventBuffer;
use webxr_api::Floor;
use webxr_api::Frame;
use webxr_api::GLContexts;
use webxr_api::InputId;
use webxr_api::InputSource;
use webxr_api::LayerGrandManager;
use webxr_api::LayerId;
use webxr_api::LayerInit;
use webxr_api::LayerManager;
use webxr_api::LayerManagerAPI;
use webxr_api::LeftEye;
use webxr_api::Native;
use webxr_api::Quitter;
use webxr_api::RightEye;
use webxr_api::SelectKind;
use webxr_api::Sender;
use webxr_api::Session as WebXrSession;
use webxr_api::SessionBuilder;
use webxr_api::SessionInit;
use webxr_api::SessionMode;
use webxr_api::SubImage;
use webxr_api::SubImages;
use webxr_api::View;
use webxr_api::ViewerPose;
use webxr_api::Viewport;
use webxr_api::Viewports;
use webxr_api::Views;
use webxr_api::Visibility;

mod input;
use input::OpenXRInput;
mod graphics;
mod interaction_profiles;
use graphics::{GraphicsProvider, GraphicsProviderMethods};

#[cfg(target_os = "windows")]
mod graphics_d3d11;
#[cfg(target_os = "windows")]
use graphics_d3d11::Backend;

const HEIGHT: f32 = 1.4;

const IDENTITY_POSE: Posef = Posef {
    orientation: Quaternionf {
        x: 0.,
        y: 0.,
        z: 0.,
        w: 1.,
    },
    position: Vector3f {
        x: 0.,
        y: 0.,
        z: 0.,
    },
};

const VIEW_INIT: openxr::View = openxr::View {
    pose: IDENTITY_POSE,
    fov: Fovf {
        angle_left: 0.,
        angle_right: 0.,
        angle_up: 0.,
        angle_down: 0.,
    },
};

// How much to downscale the view capture by.
// This is used for performance reasons, to dedicate less texture memory to the camera.
// Note that on an HL2 this allocates enough texture memory for "low power" mode,
// not "high quality" (in the device portal under
// Views > Mixed Reality Capture > Photo and Video Settings).
const SECONDARY_VIEW_DOWNSCALE: i32 = 2;

/// Provides a way to spawn and interact with context menus
pub trait ContextMenuProvider: Send {
    /// Open a context menu, return a way to poll for the result
    fn open_context_menu(&self) -> Box<dyn ContextMenuFuture>;
    /// Clone self as a trait object
    fn clone_object(&self) -> Box<dyn ContextMenuProvider>;
}

/// A way to poll for the result of the context menu request
pub trait ContextMenuFuture {
    fn poll(&self) -> ContextMenuResult;
}

/// The result of polling on a context menu request
pub enum ContextMenuResult {
    /// Session should exit
    ExitSession,
    /// Dialog was dismissed
    Dismissed,
    /// User has not acted on dialog
    Pending,
}

#[derive(Default)]
pub struct AppInfo {
    application_name: String,
    application_version: u32,
    engine_name: String,
    engine_version: u32,
}

impl AppInfo {
    pub fn new(
        application_name: &str,
        application_version: u32,
        engine_name: &str,
        engine_version: u32,
    ) -> AppInfo {
        Self {
            application_name: application_name.to_string(),
            application_version,
            engine_name: engine_name.to_string(),
            engine_version,
        }
    }
}

struct ViewInfo<Eye> {
    view: openxr::View,
    extent: Extent2Di,
    cached_projection: Transform3D<f32, Eye, Display>,
}

impl<Eye> ViewInfo<Eye> {
    fn set_view(&mut self, view: openxr::View, clip_planes: ClipPlanes) {
        self.view.pose = view.pose;
        if self.view.fov.angle_left != view.fov.angle_left
            || self.view.fov.angle_right != view.fov.angle_right
            || self.view.fov.angle_up != view.fov.angle_up
            || self.view.fov.angle_down != view.fov.angle_down
        {
            // It's fine if this happens occasionally, but if this happening very
            // often we should stop caching
            warn!("FOV changed, updating projection matrices");
            self.view.fov = view.fov;
            self.recompute_projection(clip_planes);
        }
    }

    fn recompute_projection(&mut self, clip_planes: ClipPlanes) {
        self.cached_projection = fov_to_projection_matrix(&self.view.fov, clip_planes);
    }

    fn view(&self) -> View<Eye> {
        View {
            transform: transform(&self.view.pose),
            projection: self.cached_projection,
        }
    }
}

pub struct OpenXrDiscovery {
    context_menu_provider: Option<Box<dyn ContextMenuProvider>>,
    app_info: AppInfo,
}

impl OpenXrDiscovery {
    pub fn new(
        context_menu_provider: Option<Box<dyn ContextMenuProvider>>,
        app_info: AppInfo,
    ) -> Self {
        Self {
            context_menu_provider,
            app_info,
        }
    }
}

pub struct CreatedInstance {
    instance: Instance,
    supports_hands: bool,
    supports_secondary: bool,
    system: SystemId,
    supports_mutable_fov: bool,
    supported_interaction_profiles: Vec<&'static str>,
    supports_passthrough: bool,
    supports_updating_framerate: bool,
}

pub fn create_instance(
    needs_hands: bool,
    needs_secondary: bool,
    needs_passthrough: bool,
    app_info: &AppInfo,
) -> Result<CreatedInstance, String> {
    let entry = unsafe { Entry::load().map_err(|e| format!("Entry::load {:?}", e))? };
    let supported = entry
        .enumerate_extensions()
        .map_err(|e| format!("Entry::enumerate_extensions {:?}", e))?;
    warn!("Available extensions:\n{:?}", supported);
    let mut supports_hands = needs_hands && supported.ext_hand_tracking;
    let supports_passthrough = needs_passthrough && supported.fb_passthrough;
    let supports_secondary = needs_secondary
        && supported.msft_secondary_view_configuration
        && supported.msft_first_person_observer;
    let supports_updating_framerate = supported.fb_display_refresh_rate;

    let app_info = ApplicationInfo {
        application_name: &app_info.application_name,
        application_version: app_info.application_version,
        engine_name: &app_info.engine_name,
        engine_version: app_info.engine_version,
        api_version: Version::new(1, 0, 36),
    };

    let mut exts = ExtensionSet::default();
    GraphicsProvider::enable_graphics_extensions(&mut exts);
    if supports_hands {
        exts.ext_hand_tracking = true;
    }

    if supports_secondary {
        exts.msft_secondary_view_configuration = true;
        exts.msft_first_person_observer = true;
    }

    if supports_passthrough {
        exts.fb_passthrough = true;
    }

    if supports_updating_framerate {
        exts.fb_display_refresh_rate = true;
    }

    let supported_interaction_profiles = get_supported_interaction_profiles(&supported, &mut exts);

    let instance = entry
        .create_instance(&app_info, &exts, &[])
        .map_err(|e| format!("Entry::create_instance {:?}", e))?;
    let system = instance
        .system(FormFactor::HEAD_MOUNTED_DISPLAY)
        .map_err(|e| format!("Instance::system {:?}", e))?;

    if supports_hands {
        supports_hands |= instance
            .supports_hand_tracking(system)
            .map_err(|e| format!("Instance::supports_hand_tracking {:?}", e))?;
    }

    let supports_mutable_fov = {
        let properties = instance
            .view_configuration_properties(system, ViewConfigurationType::PRIMARY_STEREO)
            .map_err(|e| format!("Instance::view_configuration_properties {:?}", e))?;
        // Unfortunately we need to do a platform check here as just flipping the FOVs for the
        // composition layer is seemingly no longer sufficient. As long as windows sessions are
        // solely backed by D3D11, we'll need to apply the same inverted view + reversed winding
        // fix for SteamVR as well as Oculus.
        properties.fov_mutable && !cfg!(target_os = "windows")
    };

    Ok(CreatedInstance {
        instance,
        supports_hands,
        supports_secondary,
        system,
        supports_mutable_fov,
        supported_interaction_profiles,
        supports_passthrough,
        supports_updating_framerate,
    })
}

impl DiscoveryAPI<SurfmanGL> for OpenXrDiscovery {
    fn request_session(
        &mut self,
        mode: SessionMode,
        init: &SessionInit,
        xr: SessionBuilder<SurfmanGL>,
    ) -> Result<WebXrSession, Error> {
        if self.supports_session(mode) {
            let needs_hands = init.feature_requested("hand-tracking");
            let needs_secondary =
                init.feature_requested("secondary-views") && init.first_person_observer_view;
            let needs_passthrough = mode == SessionMode::ImmersiveAR;
            let instance = create_instance(
                needs_hands,
                needs_secondary,
                needs_passthrough,
                &self.app_info,
            )
            .map_err(|e| Error::BackendSpecific(e))?;

            let mut supported_features = vec!["local-floor".into(), "bounded-floor".into()];
            if instance.supports_hands {
                supported_features.push("hand-tracking".into());
            }
            if instance.supports_secondary && init.first_person_observer_view {
                supported_features.push("secondary-views".into());
            }
            let granted_features = init.validate(mode, &supported_features)?;
            let context_menu_provider = self.context_menu_provider.take();
            xr.spawn(move |grand_manager| {
                OpenXrDevice::new(
                    instance,
                    granted_features,
                    context_menu_provider,
                    grand_manager,
                )
            })
        } else {
            Err(Error::NoMatchingDevice)
        }
    }

    fn supports_session(&self, mode: SessionMode) -> bool {
        let mut supports = false;
        // Determining AR support requires enumerating environment blend modes,
        // but this requires an already created XrInstance and SystemId.
        // We'll make a "default" instance here to check the blend modes,
        // then a proper one in request_session with hands/secondary support if needed.
        let needs_passthrough = mode == SessionMode::ImmersiveAR;
        if let Ok(instance) = create_instance(false, false, needs_passthrough, &self.app_info) {
            if let Ok(blend_modes) = instance.instance.enumerate_environment_blend_modes(
                instance.system,
                ViewConfigurationType::PRIMARY_STEREO,
            ) {
                if mode == SessionMode::ImmersiveAR {
                    supports = blend_modes.contains(&EnvironmentBlendMode::ADDITIVE)
                        || blend_modes.contains(&EnvironmentBlendMode::ALPHA_BLEND)
                        || instance.supports_passthrough;
                } else if mode == SessionMode::ImmersiveVR {
                    // Immersive VR sessions are not precluded by non-opaque blending
                    supports = blend_modes.len() > 0;
                }
            }
        }
        supports
    }
}

struct OpenXrDevice {
    session: Arc<Session<Backend>>,
    instance: Instance,
    events: EventBuffer,
    frame_waiter: FrameWaiter,
    layer_manager: LayerManager,
    viewer_space: Space,
    shared_data: Arc<Mutex<Option<SharedData>>>,
    clip_planes: ClipPlanes,
    supports_secondary: bool,
    supports_mutable_fov: bool,
    supports_updating_framerate: bool,

    // input
    action_set: ActionSet,
    right_hand: OpenXRInput,
    left_hand: OpenXRInput,
    granted_features: Vec<String>,
    context_menu_provider: Option<Box<dyn ContextMenuProvider>>,
    context_menu_future: Option<Box<dyn ContextMenuFuture>>,
}

/// Data that is shared between the openxr thread and the
/// layer manager that runs in the webgl thread.
struct SharedData {
    left: ViewInfo<LeftEye>,
    right: ViewInfo<RightEye>,
    secondary: Option<ViewInfo<Capture>>,
    secondary_active: bool,
    primary_blend_mode: EnvironmentBlendMode,
    secondary_blend_mode: Option<EnvironmentBlendMode>,
    frame_state: Option<FrameState>,
    space: Space,
    swapchain_sample_count: u32,
}

struct OpenXrLayerManager {
    session: Arc<Session<Backend>>,
    shared_data: Arc<Mutex<Option<SharedData>>>,
    frame_stream: FrameStream<Backend>,
    layers: Vec<(ContextId, LayerId)>,
    openxr_layers: HashMap<LayerId, OpenXrLayer>,
    clearer: GlClearer,
    _passthrough: Option<Passthrough>,
    passthrough_layer: Option<PassthroughLayer>,
}

struct OpenXrLayer {
    swapchain: Swapchain<Backend>,
    depth_stencil_texture: Option<gl::NativeTexture>,
    size: Size2D<i32, Viewport>,
    images: Vec<<Backend as Graphics>::SwapchainImage>,
    surface_textures: Vec<Option<SurfaceTexture>>,
    waited: bool,
}

impl OpenXrLayerManager {
    fn new(
        session: Arc<Session<Backend>>,
        shared_data: Arc<Mutex<Option<SharedData>>>,
        frame_stream: FrameStream<Backend>,
        should_reverse_winding: bool,
        _passthrough: Option<Passthrough>,
        passthrough_layer: Option<PassthroughLayer>,
    ) -> OpenXrLayerManager {
        let layers = Vec::new();
        let openxr_layers = HashMap::new();
        let clearer = GlClearer::new(should_reverse_winding);
        OpenXrLayerManager {
            session,
            shared_data,
            frame_stream,
            layers,
            openxr_layers,
            clearer,
            _passthrough,
            passthrough_layer,
        }
    }
}

impl OpenXrLayer {
    fn new(
        swapchain: Swapchain<Backend>,
        depth_stencil_texture: Option<gl::NativeTexture>,
        size: Size2D<i32, Viewport>,
    ) -> Result<OpenXrLayer, Error> {
        let images = swapchain
            .enumerate_images()
            .map_err(|e| Error::BackendSpecific(format!("Session::enumerate_images {:?}", e)))?;
        let waited = false;
        let mut surface_textures = Vec::new();
        surface_textures.resize_with(images.len(), || None);
        Ok(OpenXrLayer {
            swapchain,
            depth_stencil_texture,
            size,
            images,
            surface_textures,
            waited,
        })
    }

    fn get_surface_texture(
        &mut self,
        device: &mut SurfmanDevice,
        context: &mut SurfmanContext,
        index: usize,
    ) -> Result<&SurfaceTexture, SurfmanError> {
        let result = self
            .surface_textures
            .get_mut(index)
            .ok_or(SurfmanError::Failed)?;
        if let Some(result) = result {
            return Ok(result);
        }
        let surface_texture = GraphicsProvider::surface_texture_from_swapchain_texture(
            self.images[index],
            device,
            context,
            &self.size.to_untyped(),
        )?;
        *result = Some(surface_texture);
        result.as_ref().ok_or(SurfmanError::Failed)
    }
}

impl LayerManagerAPI<SurfmanGL> for OpenXrLayerManager {
    fn create_layer(
        &mut self,
        device: &mut SurfmanDevice,
        contexts: &mut dyn GLContexts<SurfmanGL>,
        context_id: ContextId,
        init: LayerInit,
    ) -> Result<LayerId, Error> {
        let guard = self.shared_data.lock().unwrap();
        let data = guard.as_ref().unwrap();

        // XXXManishearth should we be doing this, or letting Servo set the format?
        let formats = self.session.enumerate_swapchain_formats().map_err(|e| {
            Error::BackendSpecific(format!("Session::enumerate_swapchain_formats {:?}", e))
        })?;
        let format = GraphicsProvider::pick_format(&formats);
        let texture_size = init.texture_size(&data.viewports());
        let sample_count = data.swapchain_sample_count;
        let swapchain_create_info = SwapchainCreateInfo {
            create_flags: SwapchainCreateFlags::EMPTY,
            usage_flags: SwapchainUsageFlags::COLOR_ATTACHMENT | SwapchainUsageFlags::SAMPLED,
            width: texture_size.width as u32,
            height: texture_size.height as u32,
            format,
            sample_count,
            face_count: 1,
            array_size: 1,
            mip_count: 1,
        };
        let swapchain = self
            .session
            .create_swapchain(&swapchain_create_info)
            .map_err(|e| Error::BackendSpecific(format!("Session::create_swapchain {:?}", e)))?;

        // TODO: Treat depth and stencil separately?
        // TODO: Use the openxr API for depth/stencil swap chains?
        let has_depth_stencil = match init {
            LayerInit::WebGLLayer { stencil, depth, .. } => stencil | depth,
            LayerInit::ProjectionLayer { stencil, depth, .. } => stencil | depth,
        };
        let depth_stencil_texture = if has_depth_stencil {
            let gl = contexts
                .bindings(device, context_id)
                .ok_or(Error::NoMatchingDevice)?;
            unsafe {
                let depth_stencil_texture = gl.create_texture().ok();
                gl.bind_texture(gl::TEXTURE_2D, depth_stencil_texture);
                gl.tex_image_2d(
                    gl::TEXTURE_2D,
                    0,
                    gl::DEPTH24_STENCIL8 as _,
                    texture_size.width,
                    texture_size.height,
                    0,
                    gl::DEPTH_STENCIL,
                    gl::UNSIGNED_INT_24_8,
                    PixelUnpackData::Slice(None),
                );
                depth_stencil_texture
            }
        } else {
            None
        };

        let layer_id = LayerId::new();
        let openxr_layer = OpenXrLayer::new(swapchain, depth_stencil_texture, texture_size)?;
        self.layers.push((context_id, layer_id));
        self.openxr_layers.insert(layer_id, openxr_layer);
        Ok(layer_id)
    }

    fn destroy_layer(
        &mut self,
        device: &mut SurfmanDevice,
        contexts: &mut dyn GLContexts<SurfmanGL>,
        context_id: ContextId,
        layer_id: LayerId,
    ) {
        self.clearer
            .destroy_layer(device, contexts, context_id, layer_id);
        self.layers.retain(|&ids| ids != (context_id, layer_id));
        if let Some(mut layer) = self.openxr_layers.remove(&layer_id) {
            if let Some(depth_stencil_texture) = layer.depth_stencil_texture {
                let gl = contexts.bindings(device, context_id).unwrap();
                unsafe { gl.delete_texture(depth_stencil_texture) };
            }
            let mut context = contexts
                .context(device, context_id)
                .expect("missing GL context");
            for surface_texture in mem::replace(&mut layer.surface_textures, vec![]) {
                if let Some(surface_texture) = surface_texture {
                    let mut surface = device
                        .destroy_surface_texture(&mut context, surface_texture)
                        .unwrap();
                    device.destroy_surface(&mut context, &mut surface).unwrap();
                }
            }
        }
    }

    fn layers(&self) -> &[(ContextId, LayerId)] {
        &self.layers[..]
    }

    fn end_frame(
        &mut self,
        _device: &mut SurfmanDevice,
        _contexts: &mut dyn GLContexts<SurfmanGL>,
        layers: &[(ContextId, LayerId)],
    ) -> Result<(), Error> {
        let guard = self.shared_data.lock().unwrap();
        let data = guard.as_ref().unwrap();

        // At this point the frame contents have been rendered, so we can release access to the texture
        // in preparation for displaying it.
        for (_, openxr_layer) in &mut self.openxr_layers {
            if openxr_layer.waited {
                openxr_layer.swapchain.release_image().map_err(|e| {
                    Error::BackendSpecific(format!("Session::release_image {:?}", e))
                })?;
                openxr_layer.waited = false;
            }
        }

        let openxr_layers = &self.openxr_layers;

        // Invert the up/down angles so that openxr flips the texture in the y axis.
        // Additionally, swap between the L/R views to compensate for inverted up/down FOVs.
        // This has no effect in runtimes that don't support fovMutable
        let mut l_fov = data.left.view.fov;
        let mut r_fov = data.right.view.fov;
        if cfg!(target_os = "windows") {
            std::mem::swap(&mut l_fov.angle_up, &mut r_fov.angle_down);
            std::mem::swap(&mut r_fov.angle_up, &mut l_fov.angle_down);
        }

        let viewports = data.viewports();
        let primary_views = layers
            .iter()
            .filter_map(|&(_, layer_id)| {
                let openxr_layer = openxr_layers.get(&layer_id)?;
                Some([
                    openxr::CompositionLayerProjectionView::new()
                        .pose(data.left.view.pose)
                        .fov(l_fov)
                        .sub_image(
                            openxr::SwapchainSubImage::new()
                                .swapchain(&openxr_layer.swapchain)
                                .image_array_index(0)
                                .image_rect(image_rect(viewports.viewports[0])),
                        ),
                    openxr::CompositionLayerProjectionView::new()
                        .pose(data.right.view.pose)
                        .fov(r_fov)
                        .sub_image(
                            openxr::SwapchainSubImage::new()
                                .swapchain(&openxr_layer.swapchain)
                                .image_array_index(0)
                                .image_rect(image_rect(viewports.viewports[1])),
                        ),
                ])
            })
            .collect::<Vec<_>>();

        let primary_layers = primary_views
            .iter()
            .map(|views| {
                CompositionLayerProjection::new()
                    .space(&data.space)
                    .layer_flags(CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA)
                    .views(&views[..])
            })
            .collect::<Vec<_>>();

        let mut primary_layers = primary_layers
            .iter()
            .map(|layer| layer.deref())
            .collect::<Vec<_>>();

        if let Some(passthrough_layer) = &self.passthrough_layer {
            let clp = CompositionLayerPassthroughFB {
                ty: CompositionLayerPassthroughFB::TYPE,
                next: std::ptr::null(),
                flags: CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA,
                space: openxr::sys::Space::from_raw(0),
                layer_handle: *passthrough_layer.inner(),
            };
            let passthrough_base = &clp as *const _ as *const CompositionLayerBase<Backend>;
            unsafe {
                primary_layers.insert(0, &*passthrough_base);
            }
        }

        if let (Some(secondary), true) = (data.secondary.as_ref(), data.secondary_active) {
            let mut s_fov = secondary.view.fov;
            std::mem::swap(&mut s_fov.angle_up, &mut s_fov.angle_down);
            let secondary_views = layers
                .iter()
                .filter_map(|&(_, layer_id)| {
                    let openxr_layer = openxr_layers.get(&layer_id)?;
                    Some([openxr::CompositionLayerProjectionView::new()
                        .pose(secondary.view.pose)
                        .fov(s_fov)
                        .sub_image(
                            openxr::SwapchainSubImage::new()
                                .swapchain(&openxr_layer.swapchain)
                                .image_array_index(0)
                                .image_rect(image_rect(viewports.viewports[2])),
                        )])
                })
                .collect::<Vec<_>>();

            let secondary_layers = secondary_views
                .iter()
                .map(|views| {
                    CompositionLayerProjection::new()
                        .space(&data.space)
                        .layer_flags(CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA)
                        .views(&views[..])
                })
                .collect::<Vec<_>>();

            let secondary_layers = secondary_layers
                .iter()
                .map(|layer| layer.deref())
                .collect::<Vec<_>>();

            self.frame_stream
                .end_secondary(
                    data.frame_state.as_ref().unwrap().predicted_display_time,
                    data.primary_blend_mode,
                    &primary_layers[..],
                    SecondaryEndInfo {
                        ty: ViewConfigurationType::SECONDARY_MONO_FIRST_PERSON_OBSERVER_MSFT,
                        // XXXManishearth should we use the secondary layer's blend mode here, given
                        // that the content will be using the primary blend mode?
                        environment_blend_mode: data
                            .secondary_blend_mode
                            .unwrap_or(data.primary_blend_mode),
                        layers: &secondary_layers[..],
                    },
                )
                .map_err(|e| {
                    Error::BackendSpecific(format!("FrameStream::end_secondary {:?}", e))
                })?;
        } else {
            self.frame_stream
                .end(
                    data.frame_state.as_ref().unwrap().predicted_display_time,
                    data.primary_blend_mode,
                    &primary_layers[..],
                )
                .map_err(|e| Error::BackendSpecific(format!("FrameStream::end {:?}", e)))?;
        }
        Ok(())
    }

    fn begin_frame(
        &mut self,
        device: &mut SurfmanDevice,
        contexts: &mut dyn GLContexts<SurfmanGL>,
        layers: &[(ContextId, LayerId)],
    ) -> Result<Vec<SubImages>, Error> {
        let data_guard = self.shared_data.lock().unwrap();
        let data = data_guard.as_ref().unwrap();
        let openxr_layers = &mut self.openxr_layers;
        let clearer = &mut self.clearer;
        self.frame_stream
            .begin()
            .map_err(|e| Error::BackendSpecific(format!("FrameStream::begin {:?}", e)))?;
        layers
            .iter()
            .map(|&(context_id, layer_id)| {
                let context = contexts
                    .context(device, context_id)
                    .ok_or(Error::NoMatchingDevice)?;
                let openxr_layer = openxr_layers
                    .get_mut(&layer_id)
                    .ok_or(Error::NoMatchingDevice)?;

                let image = openxr_layer.swapchain.acquire_image().map_err(|e| {
                    Error::BackendSpecific(format!("Swapchain::acquire_image {:?}", e))
                })?;
                openxr_layer
                    .swapchain
                    .wait_image(openxr::Duration::INFINITE)
                    .map_err(|e| {
                        Error::BackendSpecific(format!("Swapchain::wait_image {:?}", e))
                    })?;
                openxr_layer.waited = true;

                let color_surface_texture = openxr_layer
                    .get_surface_texture(device, context, image as usize)
                    .map_err(|e| {
                        Error::BackendSpecific(format!("Layer::get_surface_texture {:?}", e))
                    })?;
                let color_texture = device.surface_texture_object(color_surface_texture);
                let color_target = device.surface_gl_texture_target();
                let depth_stencil_texture = openxr_layer
                    .depth_stencil_texture
                    .map(|texture| texture.0.get());
                let texture_array_index = None;
                let origin = Point2D::new(0, 0);
                let texture_size = openxr_layer.size;
                let sub_image = Some(SubImage {
                    color_texture,
                    depth_stencil_texture,
                    texture_array_index,
                    viewport: Rect::new(origin, texture_size),
                });
                let view_sub_images = data
                    .viewports()
                    .viewports
                    .iter()
                    .map(|&viewport| SubImage {
                        color_texture,
                        depth_stencil_texture,
                        texture_array_index,
                        viewport,
                    })
                    .collect();
                clearer.clear(
                    device,
                    contexts,
                    context_id,
                    layer_id,
                    NonZeroU32::new(color_texture).map(glow::NativeTexture),
                    color_target,
                    openxr_layer.depth_stencil_texture,
                );
                Ok(SubImages {
                    layer_id,
                    sub_image,
                    view_sub_images,
                })
            })
            .collect()
    }
}

fn image_rect(viewport: Rect<i32, Viewport>) -> openxr::Rect2Di {
    openxr::Rect2Di {
        extent: openxr::Extent2Di {
            height: viewport.size.height,
            width: viewport.size.width,
        },
        offset: openxr::Offset2Di {
            x: viewport.origin.x,
            y: viewport.origin.y,
        },
    }
}

impl OpenXrDevice {
    fn new(
        instance: CreatedInstance,
        granted_features: Vec<String>,
        context_menu_provider: Option<Box<dyn ContextMenuProvider>>,
        grand_manager: LayerGrandManager<SurfmanGL>,
    ) -> Result<OpenXrDevice, Error> {
        let CreatedInstance {
            instance,
            supports_hands,
            supports_secondary,
            system,
            supports_mutable_fov,
            supported_interaction_profiles,
            supports_passthrough,
            supports_updating_framerate,
        } = instance;

        let (init_tx, init_rx) = crossbeam_channel::unbounded();

        let instance_clone = instance.clone();
        let shared_data = Arc::new(Mutex::new(None));
        let shared_data_clone = shared_data.clone();
        let mut data = shared_data.lock().unwrap();

        let layer_manager = grand_manager.create_layer_manager(move |device, _| {
            let (session, frame_waiter, frame_stream) =
                GraphicsProvider::create_session(device, &instance_clone, system)?;
            let (passthrough, passthrough_layer) = if supports_passthrough {
                let flags = PassthroughFlagsFB::IS_RUNNING_AT_CREATION;
                let purpose = PassthroughLayerPurposeFB::RECONSTRUCTION;
                let passthrough = session
                    .create_passthrough(flags)
                    .expect("Unable to initialize passthrough");
                let passthrough_layer = session
                    .create_passthrough_layer(&passthrough, flags, purpose)
                    .expect("Failed to create passthrough layer");
                (Some(passthrough), Some(passthrough_layer))
            } else {
                (None, None)
            };
            let session = Arc::new(session);
            init_tx
                .send((session.clone(), frame_waiter))
                .map_err(|_| Error::CommunicationError)?;
            Ok(OpenXrLayerManager::new(
                session,
                shared_data_clone,
                frame_stream,
                !supports_mutable_fov,
                passthrough,
                passthrough_layer,
            ))
        })?;

        let (session, frame_waiter) = init_rx.recv().map_err(|_| Error::CommunicationError)?;

        // XXXPaul initialisation should happen on SessionStateChanged(Ready)?

        if supports_secondary {
            session
                .begin_with_secondary(
                    ViewConfigurationType::PRIMARY_STEREO,
                    &[ViewConfigurationType::SECONDARY_MONO_FIRST_PERSON_OBSERVER_MSFT],
                )
                .map_err(|e| {
                    Error::BackendSpecific(format!("Session::begin_with_secondary {:?}", e))
                })?;
        } else {
            session
                .begin(ViewConfigurationType::PRIMARY_STEREO)
                .map_err(|e| Error::BackendSpecific(format!("Session::begin {:?}", e)))?;
        }

        let pose = Posef {
            orientation: Quaternionf {
                x: 0.,
                y: 0.,
                z: 0.,
                w: 1.,
            },
            position: Vector3f {
                x: 0.,
                y: 0.,
                z: 0.,
            },
        };
        let space = session
            .create_reference_space(ReferenceSpaceType::LOCAL, pose)
            .map_err(|e| {
                Error::BackendSpecific(format!("Session::create_reference_space {:?}", e))
            })?;

        let viewer_space = session
            .create_reference_space(ReferenceSpaceType::VIEW, pose)
            .map_err(|e| {
                Error::BackendSpecific(format!("Session::create_reference_space {:?}", e))
            })?;

        let view_configuration_type = ViewConfigurationType::PRIMARY_STEREO;
        let view_configurations = instance
            .enumerate_view_configuration_views(system, view_configuration_type)
            .map_err(|e| {
                Error::BackendSpecific(format!(
                    "Session::enumerate_view_configuration_views {:?}",
                    e
                ))
            })?;

        let left_view_configuration = view_configurations[0];
        let right_view_configuration = view_configurations[1];
        let left_extent = Extent2Di {
            width: left_view_configuration.recommended_image_rect_width as i32,
            height: left_view_configuration.recommended_image_rect_height as i32,
        };
        let right_extent = Extent2Di {
            width: right_view_configuration.recommended_image_rect_width as i32,
            height: right_view_configuration.recommended_image_rect_height as i32,
        };

        assert_eq!(
            left_view_configuration.recommended_image_rect_height,
            right_view_configuration.recommended_image_rect_height,
        );

        let swapchain_sample_count = left_view_configuration.recommended_swapchain_sample_count;

        let secondary_active = false;
        let (secondary, secondary_blend_mode) = if supports_secondary {
            let view_configuration = *instance
                .enumerate_view_configuration_views(
                    system,
                    ViewConfigurationType::SECONDARY_MONO_FIRST_PERSON_OBSERVER_MSFT,
                )
                .map_err(|e| {
                    Error::BackendSpecific(format!(
                        "Session::enumerate_view_configuration_views {:?}",
                        e
                    ))
                })?
                .get(0)
                .expect(
                    "Session::enumerate_view_configuration_views() returned no secondary views",
                );

            let secondary_blend_mode = instance
                .enumerate_environment_blend_modes(
                    system,
                    ViewConfigurationType::SECONDARY_MONO_FIRST_PERSON_OBSERVER_MSFT,
                )
                .map_err(|e| {
                    Error::BackendSpecific(format!(
                        "Instance::enumerate_environment_blend_modes {:?}",
                        e
                    ))
                })?[0];

            let secondary_extent = Extent2Di {
                width: view_configuration.recommended_image_rect_width as i32,
                height: view_configuration.recommended_image_rect_height as i32,
            };

            let secondary = ViewInfo {
                view: VIEW_INIT,
                extent: secondary_extent,
                cached_projection: Transform3D::identity(),
            };

            (Some(secondary), Some(secondary_blend_mode))
        } else {
            (None, None)
        };

        let primary_blend_mode = instance
            .enumerate_environment_blend_modes(system, view_configuration_type)
            .map_err(|e| {
                Error::BackendSpecific(format!(
                    "Instance::enumerate_environment_blend_modes {:?}",
                    e
                ))
            })?[0];

        let left = ViewInfo {
            view: VIEW_INIT,
            extent: left_extent,
            cached_projection: Transform3D::identity(),
        };
        let right = ViewInfo {
            view: VIEW_INIT,
            extent: right_extent,
            cached_projection: Transform3D::identity(),
        };
        *data = Some(SharedData {
            frame_state: None,
            space,
            left,
            right,
            secondary,
            secondary_active,
            primary_blend_mode,
            secondary_blend_mode,
            swapchain_sample_count,
        });
        drop(data);

        let (action_set, right_hand, left_hand) = OpenXRInput::setup_inputs(
            &instance,
            &session,
            supports_hands,
            supported_interaction_profiles,
        );

        Ok(OpenXrDevice {
            instance,
            events: Default::default(),
            session,
            frame_waiter,
            viewer_space,
            clip_planes: Default::default(),
            supports_secondary,
            supports_mutable_fov,
            supports_updating_framerate,
            layer_manager,
            shared_data,

            action_set,
            right_hand,
            left_hand,
            granted_features,
            context_menu_provider,
            context_menu_future: None,
        })
    }

    fn handle_openxr_events(&mut self) -> bool {
        use openxr::Event::*;
        let mut stopped = false;
        loop {
            let mut buffer = openxr::EventDataBuffer::new();
            let event = match self.instance.poll_event(&mut buffer) {
                Ok(event) => event,
                Err(e) => {
                    error!("Error polling events: {:?}", e);
                    return false;
                }
            };
            match event {
                Some(SessionStateChanged(session_change)) => match session_change.state() {
                    openxr::SessionState::EXITING | openxr::SessionState::LOSS_PENDING => {
                        self.events.callback(Event::SessionEnd);
                        return false;
                    }
                    openxr::SessionState::STOPPING => {
                        self.events
                            .callback(Event::VisibilityChange(Visibility::Hidden));
                        if let Err(e) = self.session.end() {
                            error!("Session failed to end on STOPPING: {:?}", e);
                        }
                        stopped = true;
                    }
                    openxr::SessionState::READY if stopped => {
                        self.events
                            .callback(Event::VisibilityChange(Visibility::Visible));
                        if let Err(e) = self.session.begin(ViewConfigurationType::PRIMARY_STEREO) {
                            error!("Session failed to begin on READY: {:?}", e);
                        }
                        stopped = false;
                    }
                    openxr::SessionState::FOCUSED => {
                        self.events
                            .callback(Event::VisibilityChange(Visibility::Visible));
                    }
                    openxr::SessionState::VISIBLE => {
                        self.events
                            .callback(Event::VisibilityChange(Visibility::VisibleBlurred));
                    }
                    _ => {
                        // FIXME: Handle other states
                    }
                },
                Some(InstanceLossPending(_)) => {
                    self.events.callback(Event::SessionEnd);
                    return false;
                }
                Some(InteractionProfileChanged(_)) => {
                    let path = self.instance.string_to_path("/user/hand/right").unwrap();
                    let profile_path = self.session.current_interaction_profile(path).unwrap();
                    let profile = self.instance.path_to_string(profile_path);

                    match profile {
                        Ok(profile) => {
                            let profiles = get_profiles_from_path(profile)
                                .iter()
                                .map(|s| s.to_string())
                                .collect();

                            let mut new_left = self.left_hand.input_source();
                            new_left.profiles.clone_from(&profiles);
                            self.events
                                .callback(Event::UpdateInput(new_left.id, new_left));

                            let mut new_right = self.right_hand.input_source();
                            new_right.profiles.clone_from(&profiles);
                            self.events
                                .callback(Event::UpdateInput(new_right.id, new_right));
                        }
                        Err(e) => {
                            error!("Failed to get interaction profile: {:?}", e);
                        }
                    }
                }
                Some(ReferenceSpaceChangePending(e)) => {
                    let base_space = match e.reference_space_type() {
                        ReferenceSpaceType::VIEW => BaseSpace::Viewer,
                        ReferenceSpaceType::LOCAL => BaseSpace::Local,
                        ReferenceSpaceType::LOCAL_FLOOR => BaseSpace::Floor,
                        ReferenceSpaceType::STAGE => BaseSpace::BoundedFloor,
                        _ => unreachable!(
                            "Should not be receiving change events for unsupported space types"
                        ),
                    };
                    let transform = transform(&e.pose_in_previous_space());
                    self.events
                        .callback(Event::ReferenceSpaceChanged(base_space, transform));
                }
                Some(_) => {
                    // FIXME: Handle other events
                }
                None if stopped => {
                    // XXXManishearth be able to handle exits during this time
                    thread::sleep(Duration::from_millis(200));
                }
                None => {
                    // No more events to process
                    break;
                }
            }
        }
        true
    }
}

impl SharedData {
    fn views(&self) -> Views {
        let left_view = self.left.view();
        let right_view = self.right.view();
        if let (Some(secondary), true) = (self.secondary.as_ref(), self.secondary_active) {
            // Note: we report the secondary view only when it is active
            let third_eye = secondary.view();
            return Views::StereoCapture(left_view, right_view, third_eye);
        }
        Views::Stereo(left_view, right_view)
    }

    fn viewports(&self) -> Viewports {
        let left_vp = Rect::new(
            Point2D::zero(),
            Size2D::new(self.left.extent.width, self.left.extent.height),
        );
        let right_vp = Rect::new(
            Point2D::new(self.left.extent.width, 0),
            Size2D::new(self.right.extent.width, self.right.extent.height),
        );
        let mut viewports = vec![left_vp, right_vp];
        // Note: we report the secondary viewport even when it is inactive
        if let Some(ref secondary) = self.secondary {
            let secondary_vp = Rect::new(
                Point2D::new(self.left.extent.width + self.right.extent.width, 0),
                Size2D::new(secondary.extent.width, secondary.extent.height)
                    / SECONDARY_VIEW_DOWNSCALE,
            );
            viewports.push(secondary_vp)
        }
        Viewports { viewports }
    }
}

impl DeviceAPI for OpenXrDevice {
    fn floor_transform(&self) -> Option<RigidTransform3D<f32, Native, Floor>> {
        let translation = Vector3D::new(0.0, HEIGHT, 0.0);
        Some(RigidTransform3D::from_translation(translation))
    }

    fn viewports(&self) -> Viewports {
        self.shared_data
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .viewports()
    }

    fn create_layer(&mut self, context_id: ContextId, init: LayerInit) -> Result<LayerId, Error> {
        self.layer_manager.create_layer(context_id, init)
    }

    fn destroy_layer(&mut self, context_id: ContextId, layer_id: LayerId) {
        self.layer_manager.destroy_layer(context_id, layer_id)
    }

    fn begin_animation_frame(&mut self, layers: &[(ContextId, LayerId)]) -> Option<Frame> {
        if !self.handle_openxr_events() {
            warn!("no frame, session isn't running");
            // Session is not running anymore.
            return None;
        }
        if let Some(ref context_menu_future) = self.context_menu_future {
            match context_menu_future.poll() {
                ContextMenuResult::ExitSession => {
                    self.quit();
                    return None;
                }
                ContextMenuResult::Dismissed => self.context_menu_future = None,
                ContextMenuResult::Pending => (),
            }
        }

        let (frame_state, secondary_state) = if self.supports_secondary {
            let (frame_state, secondary_state) = match self.frame_waiter.wait_secondary() {
                Ok(frame_state) => frame_state,
                Err(e) => {
                    error!("Error waiting on frame: {:?}", e);
                    return None;
                }
            };

            assert_eq!(
                secondary_state.ty,
                ViewConfigurationType::SECONDARY_MONO_FIRST_PERSON_OBSERVER_MSFT
            );
            (frame_state, Some(secondary_state))
        } else {
            match self.frame_waiter.wait() {
                Ok(frame_state) => (frame_state, None),
                Err(e) => {
                    error!("Error waiting on frame: {:?}", e);
                    return None;
                }
            }
        };

        // We get the subimages before grabbing the lock,
        // since otherwise we'll deadlock
        let sub_images = self.layer_manager.begin_frame(layers).ok()?;

        let mut guard = self.shared_data.lock().unwrap();
        let data = guard.as_mut().unwrap();

        // XXXManishearth should we check frame_state.should_render?
        let (_view_flags, mut views) = match self.session.locate_views(
            ViewConfigurationType::PRIMARY_STEREO,
            frame_state.predicted_display_time,
            &data.space,
        ) {
            Ok(data) => data,
            Err(e) => {
                error!("Error locating views: {:?}", e);
                return None;
            }
        };
        if !self.supports_mutable_fov {
            views.iter_mut().for_each(|v| {
                std::mem::swap(&mut v.fov.angle_up, &mut v.fov.angle_down);
            });
        }
        data.left.set_view(views[0], self.clip_planes);
        data.right.set_view(views[1], self.clip_planes);
        let pose = match self
            .viewer_space
            .locate(&data.space, frame_state.predicted_display_time)
        {
            Ok(pose) => pose,
            Err(e) => {
                error!("Error locating viewer space: {:?}", e);
                return None;
            }
        };
        let transform = transform(&pose.pose);

        if let Some(secondary_state) = secondary_state.as_ref() {
            data.secondary_active = secondary_state.active;
        }
        if let (Some(secondary), true) = (data.secondary.as_mut(), data.secondary_active) {
            let view = match self.session.locate_views(
                ViewConfigurationType::SECONDARY_MONO_FIRST_PERSON_OBSERVER_MSFT,
                frame_state.predicted_display_time,
                &data.space,
            ) {
                Ok(v) => v.1[0],
                Err(e) => {
                    error!("Error locating views: {:?}", e);
                    return None;
                }
            };
            secondary.set_view(view, self.clip_planes);
        }

        let active_action_set = ActiveActionSet::new(&self.action_set);

        if let Err(e) = self.session.sync_actions(&[active_action_set]) {
            error!("Error syncing actions: {:?}", e);
            return None;
        }

        let mut right = self
            .right_hand
            .frame(&self.session, &frame_state, &data.space, &transform);
        let mut left = self
            .left_hand
            .frame(&self.session, &frame_state, &data.space, &transform);

        data.frame_state = Some(frame_state);
        let views = data.views();

        if let Some(ref context_menu_provider) = self.context_menu_provider {
            if (left.menu_selected || right.menu_selected) && self.context_menu_future.is_none() {
                self.context_menu_future = Some(context_menu_provider.open_context_menu());
            } else if self.context_menu_future.is_some() {
                // Do not surface input info whilst the context menu is open
                // We don't do this for the first frame after the context menu is opened
                // so that the appropriate select cancel events may fire
                right.frame.target_ray_origin = None;
                right.frame.grip_origin = None;
                left.frame.target_ray_origin = None;
                left.frame.grip_origin = None;
                right.select = None;
                right.squeeze = None;
                left.select = None;
                left.squeeze = None;
            }
        }

        let left_input_changed = left.frame.input_changed;
        let right_input_changed = right.frame.input_changed;

        let frame = Frame {
            pose: Some(ViewerPose { transform, views }),
            inputs: vec![right.frame, left.frame],
            events: vec![],
            sub_images,
            hit_test_results: vec![],
            predicted_display_time: frame_state.predicted_display_time.as_nanos() as f64,
        };

        if let Some(right_select) = right.select {
            self.events.callback(Event::Select(
                InputId(0),
                SelectKind::Select,
                right_select,
                frame.clone(),
            ));
        }
        if let Some(right_squeeze) = right.squeeze {
            self.events.callback(Event::Select(
                InputId(0),
                SelectKind::Squeeze,
                right_squeeze,
                frame.clone(),
            ));
        }
        if let Some(left_select) = left.select {
            self.events.callback(Event::Select(
                InputId(1),
                SelectKind::Select,
                left_select,
                frame.clone(),
            ));
        }
        if let Some(left_squeeze) = left.squeeze {
            self.events.callback(Event::Select(
                InputId(1),
                SelectKind::Squeeze,
                left_squeeze,
                frame.clone(),
            ));
        }
        if left_input_changed {
            self.events
                .callback(Event::InputChanged(InputId(1), frame.inputs[1].clone()))
        }
        if right_input_changed {
            self.events
                .callback(Event::InputChanged(InputId(0), frame.inputs[0].clone()))
        }
        Some(frame)
    }

    fn end_animation_frame(&mut self, layers: &[(ContextId, LayerId)]) {
        // We tell OpenXR to display the frame in the layer manager.
        // Due to threading issues we can't call D3D11 APIs on the openxr thread as the
        // WebGL thread might be using the device simultaneously, so this method delegates
        // everything to the layer manager.
        let _ = self.layer_manager.end_frame(layers);
    }

    fn initial_inputs(&self) -> Vec<InputSource> {
        vec![
            self.right_hand.input_source(),
            self.left_hand.input_source(),
        ]
    }

    fn set_event_dest(&mut self, dest: Sender<Event>) {
        self.events.upgrade(dest)
    }

    fn quit(&mut self) {
        self.session.request_exit().unwrap();
        loop {
            let mut buffer = openxr::EventDataBuffer::new();
            let event = match self.instance.poll_event(&mut buffer) {
                Ok(e) => e,
                Err(e) => {
                    error!("Error polling for event while quitting: {:?}", e);
                    break;
                }
            };
            match event {
                Some(openxr::Event::SessionStateChanged(session_change)) => {
                    match session_change.state() {
                        openxr::SessionState::EXITING => {
                            break;
                        }
                        openxr::SessionState::STOPPING => {
                            if let Err(e) = self.session.end() {
                                error!("Session failed to end while STOPPING: {:?}", e);
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
            thread::sleep(Duration::from_millis(30));
        }
        self.events.callback(Event::SessionEnd);
        // We clear this data to remove the outstanding reference to XrSpace,
        // which keeps other OpenXR objects alive.
        *self.shared_data.lock().unwrap() = None;
    }

    fn set_quitter(&mut self, _: Quitter) {
        // the quitter is only needed if we have anything from outside the render
        // thread that can signal a quit. We don't.
    }

    fn update_clip_planes(&mut self, near: f32, far: f32) {
        self.clip_planes.update(near, far);
    }

    fn environment_blend_mode(&self) -> webxr_api::EnvironmentBlendMode {
        match self
            .shared_data
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .primary_blend_mode
        {
            EnvironmentBlendMode::OPAQUE => webxr_api::EnvironmentBlendMode::Opaque,
            EnvironmentBlendMode::ALPHA_BLEND => webxr_api::EnvironmentBlendMode::AlphaBlend,
            EnvironmentBlendMode::ADDITIVE => webxr_api::EnvironmentBlendMode::Additive,
            v => unimplemented!("unsupported blend mode: {:?}", v),
        }
    }

    fn granted_features(&self) -> &[String] {
        &self.granted_features
    }

    fn update_frame_rate(&mut self, rate: f32) -> f32 {
        if self.supports_updating_framerate {
            self.session
                .request_display_refresh_rate(rate)
                .expect("Failed to request display refresh rate");
            self.session
                .get_display_refresh_rate()
                .expect("Failed to get display refresh rate")
        } else {
            -1.0
        }
    }

    fn supported_frame_rates(&self) -> Vec<f32> {
        if self.supports_updating_framerate {
            self.session
                .enumerate_display_refresh_rates()
                .expect("Failed to enumerate display refresh rates")
        } else {
            vec![]
        }
    }

    fn reference_space_bounds(&self) -> Option<Vec<Point2D<f32, Floor>>> {
        match self
            .session
            .reference_space_bounds_rect(ReferenceSpaceType::STAGE)
        {
            Ok(bounds) => {
                if let Some(bounds) = bounds {
                    let point1 = Point2D::new(-bounds.width / 2., -bounds.height / 2.);
                    let point2 = Point2D::new(-bounds.width / 2., bounds.height / 2.);
                    let point3 = Point2D::new(bounds.width / 2., bounds.height / 2.);
                    let point4 = Point2D::new(bounds.width / 2., -bounds.height / 2.);
                    Some(vec![point1, point2, point3, point4])
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

fn transform<Src, Dst>(pose: &Posef) -> RigidTransform3D<f32, Src, Dst> {
    let rotation = Rotation3D::quaternion(
        pose.orientation.x,
        pose.orientation.y,
        pose.orientation.z,
        pose.orientation.w,
    );
    let translation = Vector3D::new(pose.position.x, pose.position.y, pose.position.z);
    RigidTransform3D::new(rotation, translation)
}

#[inline]
fn fov_to_projection_matrix<T, U>(fov: &Fovf, clip_planes: ClipPlanes) -> Transform3D<f32, T, U> {
    util::fov_to_projection_matrix(
        fov.angle_left,
        fov.angle_right,
        fov.angle_up,
        fov.angle_down,
        clip_planes,
    )
}
