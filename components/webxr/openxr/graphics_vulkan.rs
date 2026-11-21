/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Linux OpenXR Vulkan graphics backend (`XR_KHR_vulkan_enable2`) with DMA-BUF
//! `EGLImage` interop into Surfman's OpenGL/EGL WebGL contexts.
//!
//! # Runtime requirements
//! - An OpenXR runtime supporting `XR_KHR_vulkan_enable2` (for example
//!   SteamVR, Monado, or WiVRn), discoverable via standard OpenXR loader paths
//!   or `XR_RUNTIME_JSON`.
//! - A Vulkan 1.1+ driver supporting `VK_KHR_external_memory_fd` and
//!   `VK_EXT_external_memory_dma_buf` (plus optional
//!   `VK_EXT_image_drm_format_modifier` for tiled modifier negotiation, with
//!   automatic fallback to `VK_IMAGE_TILING_LINEAR`).
//! - An EGL driver supporting `EGL_KHR_image_base`,
//!   `EGL_EXT_image_dma_buf_import`, and `GL_OES_EGL_image`.
//!
//! # Rendering pipeline
//! Each WebXR layer owns one DMA-BUF exportable staging `VkImage` imported as a
//! `GL_TEXTURE_2D` in the layer's WebGL context. On `end_frame`, WebGL commands
//! are flushed (`glFinish`) and the staging image is blitted into the acquired
//! OpenXR swapchain `VkImage` with a hardware Y-flip (`vkCmdBlitImage`) before
//! transitioning to `VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL` and releasing.

use std::ffi::{CStr, c_char, c_void};
use std::ptr;
use std::sync::{Arc, LazyLock, Mutex};

use ash::vk::{self, Handle as _};
use euclid::{Size2D, UnknownUnit};
use glow::{self as gl, HasContext};
use log::debug;
use openxr::{
    ExtensionSet, FrameStream, FrameWaiter, Graphics, Instance, Session, SwapchainUsageFlags,
    SystemId, Version, Vulkan,
};
use surfman::{Context as SurfmanContext, Device as SurfmanDevice, Error as SurfmanError};
use webxr_api::Error;

use crate::openxr::graphics::{
    GraphicsProvider, GraphicsProviderMethods, LayerGraphics, SessionGraphics, SwapchainTexture,
};

pub type Backend = Vulkan;

const EGL_TRUE: u32 = 1;
const EGL_NONE: i32 = 0x3038;
const EGL_EXTENSIONS: i32 = 0x3055;
const EGL_HEIGHT: i32 = 0x3056;
const EGL_WIDTH: i32 = 0x3057;
const EGL_LINUX_DMA_BUF_EXT: u32 = 0x3270;
const EGL_LINUX_DRM_FOURCC_EXT: i32 = 0x3271;
const EGL_DMA_BUF_PLANE0_FD_EXT: i32 = 0x3272;
const EGL_DMA_BUF_PLANE0_OFFSET_EXT: i32 = 0x3273;
const EGL_DMA_BUF_PLANE0_PITCH_EXT: i32 = 0x3274;
const EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT: i32 = 0x3443;
const EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT: i32 = 0x3444;
// Little-endian [R, G, B, A], matching VK_FORMAT_R8G8B8A8_*.
const DRM_FORMAT_ABGR8888: u32 = u32::from_le_bytes(*b"AB24");

type EglDisplay = *mut c_void;
type EglImage = *mut c_void;

struct EglDmaBuf {
    get_current_display: unsafe extern "C" fn() -> EglDisplay,
    query_string: unsafe extern "C" fn(EglDisplay, i32) -> *const c_char,
    create_image:
        unsafe extern "C" fn(EglDisplay, *mut c_void, u32, *mut c_void, *const i32) -> EglImage,
    destroy_image: unsafe extern "C" fn(EglDisplay, EglImage) -> u32,
    query_dmabuf_modifiers:
        Option<unsafe extern "C" fn(EglDisplay, i32, i32, *mut u64, *mut u32, *mut i32) -> u32>,
    image_target_texture_2d: unsafe extern "C" fn(u32, EglImage),
}

static EGL_DMA_BUF: LazyLock<Result<EglDmaBuf, String>> = LazyLock::new(|| {
    let handle = [c"libEGL.so.1", c"libEGL.so"]
        .into_iter()
        .map(|s| unsafe { libc::dlopen(s.as_ptr(), libc::RTLD_LAZY | libc::RTLD_LOCAL) })
        .find(|h| !h.is_null())
        .ok_or_else(|| "Unable to load libEGL.so.1 or libEGL.so".to_string())?;

    let dlsym = |name: &CStr| -> Result<*mut c_void, String> {
        let sym = unsafe { libc::dlsym(handle, name.as_ptr()) };
        if sym.is_null() {
            Err(format!("Missing EGL symbol {name:?}"))
        } else {
            Ok(sym)
        }
    };
    let get_proc: unsafe extern "C" fn(*const c_char) -> *mut c_void =
        unsafe { std::mem::transmute_copy(&dlsym(c"eglGetProcAddress")?) };
    let proc_addr = |name: &CStr| -> Result<*mut c_void, String> {
        let sym = unsafe { get_proc(name.as_ptr()) };
        if sym.is_null() {
            Err(format!("Missing EGL/GL extension proc {name:?}"))
        } else {
            Ok(sym)
        }
    };

    unsafe {
        Ok(EglDmaBuf {
            get_current_display: std::mem::transmute_copy(&dlsym(c"eglGetCurrentDisplay")?),
            query_string: std::mem::transmute_copy(&dlsym(c"eglQueryString")?),
            create_image: std::mem::transmute_copy(&proc_addr(c"eglCreateImageKHR")?),
            destroy_image: std::mem::transmute_copy(&proc_addr(c"eglDestroyImageKHR")?),
            query_dmabuf_modifiers: proc_addr(c"eglQueryDmaBufModifiersEXT")
                .ok()
                .map(|p| std::mem::transmute_copy(&p)),
            image_target_texture_2d: std::mem::transmute_copy(&proc_addr(
                c"glEGLImageTargetTexture2DOES",
            )?),
        })
    }
});

struct SubmitState {
    cmd_pool: vk::CommandPool,
    cmd_buf: vk::CommandBuffer,
    fence: vk::Fence,
}

pub(super) struct VulkanContext {
    submit: Mutex<SubmitState>,
    queue: vk::Queue,
    queue_family_index: u32,
    external_memory_fd: ash::khr::external_memory_fd::Device,
    drm_format_modifier: Option<ash::ext::image_drm_format_modifier::Device>,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    instance: ash::Instance,
    // Keep libvulkan loaded until after destroy_device and destroy_instance.
    _entry: ash::Entry,
}

// Safety: `SubmitState` serializes queue/command submissions with a `Mutex`;
// Vulkan instance/device handles are thread-safe for allocation and queries.
unsafe impl Send for VulkanContext {}
unsafe impl Sync for VulkanContext {}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            if let Ok(state) = self.submit.get_mut() {
                self.device.destroy_fence(state.fence, None);
                self.device.destroy_command_pool(state.cmd_pool, None);
            }
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanContext {
    fn new(xr_instance: &Instance, system: SystemId) -> Result<Arc<Self>, Error> {
        let reqs = Vulkan::requirements(xr_instance, system)
            .map_err(|e| Error::BackendSpecific(format!("Vulkan::requirements: {e:?}")))?;
        let min_version = Version::new(1, 1, 0);
        let target_xr = if reqs.min_api_version_supported > min_version {
            reqs.min_api_version_supported
        } else {
            min_version
        };
        if target_xr.major() > reqs.max_api_version_supported.major() {
            return Err(Error::BackendSpecific(format!(
                "OpenXR Vulkan version range unsupported: min={}, max={}",
                reqs.min_api_version_supported, reqs.max_api_version_supported
            )));
        }
        let vk_api_version =
            vk::make_api_version(0, target_xr.major() as u32, target_xr.minor() as u32, 0);

        let entry = unsafe {
            ash::Entry::load()
                .map_err(|e| Error::BackendSpecific(format!("Failed to load libvulkan: {e}")))?
        };
        let get_instance_proc_addr: openxr::sys::platform::VkGetInstanceProcAddr =
            unsafe { std::mem::transmute(entry.static_fn().get_instance_proc_addr) };

        let app_info = vk::ApplicationInfo::default().api_version(vk_api_version);
        let inst_info = vk::InstanceCreateInfo::default().application_info(&app_info);
        let raw_instance = unsafe {
            xr_instance
                .create_vulkan_instance(
                    system,
                    get_instance_proc_addr,
                    (&raw const inst_info).cast(),
                )
                .map_err(|e| Error::BackendSpecific(format!("create_vulkan_instance XR: {e:?}")))?
                .map_err(|r| {
                    Error::BackendSpecific(format!(
                        "create_vulkan_instance VkResult: {:?}",
                        vk::Result::from_raw(r)
                    ))
                })?
        };
        let instance = unsafe {
            ash::Instance::load(entry.static_fn(), vk::Instance::from_raw(raw_instance as _))
        };

        let setup = Self::create_device_for_instance(
            &instance,
            xr_instance,
            system,
            get_instance_proc_addr,
            vk_api_version,
        );
        let (physical_device, device, queue_family_index, enable_drm_mod) = match setup {
            Ok(v) => v,
            Err(err) => {
                unsafe { instance.destroy_instance(None) };
                return Err(err);
            },
        };
        Self::from_handles(
            entry,
            instance,
            physical_device,
            device,
            queue_family_index,
            enable_drm_mod,
        )
    }

    fn create_device_for_instance(
        instance: &ash::Instance,
        xr_instance: &Instance,
        system: SystemId,
        get_instance_proc_addr: openxr::sys::platform::VkGetInstanceProcAddr,
        vk_api_version: u32,
    ) -> Result<(vk::PhysicalDevice, ash::Device, u32, bool), Error> {
        let raw_phys = unsafe {
            xr_instance
                .vulkan_graphics_device(system, instance.handle().as_raw() as _)
                .map_err(|e| Error::BackendSpecific(format!("vulkan_graphics_device: {e:?}")))?
        };
        let physical_device = vk::PhysicalDevice::from_raw(raw_phys as _);
        let props = unsafe { instance.get_physical_device_properties(physical_device) };
        if props.api_version < vk_api_version {
            return Err(Error::BackendSpecific(format!(
                "Vulkan physical device API {:#x} < required {:#x}",
                props.api_version, vk_api_version
            )));
        }

        let queue_family_index =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) }
                .into_iter()
                .enumerate()
                .find_map(|(i, info)| {
                    info.queue_flags
                        .contains(vk::QueueFlags::GRAPHICS)
                        .then_some(i as u32)
                })
                .ok_or_else(|| Error::BackendSpecific("No Vulkan graphics queue family".into()))?;

        let supported_exts = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .map_err(|e| {
                    Error::BackendSpecific(format!("enumerate_device_extension_properties: {e:?}"))
                })?
        };
        let has_ext = |name: &CStr| {
            supported_exts
                .iter()
                .any(|p| unsafe { CStr::from_ptr(p.extension_name.as_ptr()) } == name)
        };
        if !has_ext(ash::khr::external_memory_fd::NAME) ||
            !has_ext(ash::ext::external_memory_dma_buf::NAME)
        {
            return Err(Error::BackendSpecific(
                "Vulkan device lacks VK_KHR_external_memory_fd or VK_EXT_external_memory_dma_buf"
                    .into(),
            ));
        }

        let mut enabled_exts = vec![
            ash::khr::external_memory_fd::NAME.as_ptr(),
            ash::ext::external_memory_dma_buf::NAME.as_ptr(),
        ];
        let enable_drm_mod = has_ext(ash::ext::image_drm_format_modifier::NAME) &&
            (vk_api_version >= vk::API_VERSION_1_2 ||
                has_ext(ash::khr::image_format_list::NAME));
        if enable_drm_mod {
            enabled_exts.push(ash::ext::image_drm_format_modifier::NAME.as_ptr());
            if vk_api_version < vk::API_VERSION_1_2 {
                enabled_exts.push(ash::khr::image_format_list::NAME.as_ptr());
            }
        }

        let priorities = [1.0f32];
        let queue_infos = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)];
        let dev_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&enabled_exts);
        let raw_device = unsafe {
            xr_instance
                .create_vulkan_device(
                    system,
                    get_instance_proc_addr,
                    physical_device.as_raw() as _,
                    (&raw const dev_info).cast(),
                )
                .map_err(|e| Error::BackendSpecific(format!("create_vulkan_device XR: {e:?}")))?
                .map_err(|r| {
                    Error::BackendSpecific(format!(
                        "create_vulkan_device VkResult: {:?}",
                        vk::Result::from_raw(r)
                    ))
                })?
        };
        let device =
            unsafe { ash::Device::load(instance.fp_v1_0(), vk::Device::from_raw(raw_device as _)) };
        Ok((physical_device, device, queue_family_index, enable_drm_mod))
    }

    fn from_handles(
        entry: ash::Entry,
        instance: ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        queue_family_index: u32,
        enable_drm_mod: bool,
    ) -> Result<Arc<Self>, Error> {
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let external_memory_fd = ash::khr::external_memory_fd::Device::new(&instance, &device);
        let drm_format_modifier = enable_drm_mod
            .then(|| ash::ext::image_drm_format_modifier::Device::new(&instance, &device));

        let cmd_pool = unsafe {
            device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(queue_family_index)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )
        }
        .map_err(|e| {
            unsafe {
                device.destroy_device(None);
                instance.destroy_instance(None);
            }
            Error::BackendSpecific(format!("create_command_pool: {e:?}"))
        })?;

        let alloc_res = unsafe {
            device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(cmd_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1),
            )
        };
        let cmd_buf = match alloc_res {
            Ok(bufs) => bufs[0],
            Err(e) => {
                unsafe {
                    device.destroy_command_pool(cmd_pool, None);
                    device.destroy_device(None);
                    instance.destroy_instance(None);
                }
                return Err(Error::BackendSpecific(format!(
                    "allocate_command_buffers: {e:?}"
                )));
            },
        };

        let fence = match unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) } {
            Ok(fence) => fence,
            Err(e) => {
                unsafe {
                    device.destroy_command_pool(cmd_pool, None);
                    device.destroy_device(None);
                    instance.destroy_instance(None);
                }
                return Err(Error::BackendSpecific(format!("create_fence: {e:?}")));
            },
        };

        Ok(Arc::new(Self {
            submit: Mutex::new(SubmitState {
                cmd_pool,
                cmd_buf,
                fence,
            }),
            queue,
            queue_family_index,
            external_memory_fd,
            drm_format_modifier,
            device,
            physical_device,
            instance,
            _entry: entry,
        }))
    }

    fn find_memory_type(&self, type_bits: u32, preferred: vk::MemoryPropertyFlags) -> Option<u32> {
        let mem_props = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };
        let count = mem_props.memory_type_count as usize;
        (0..count)
            .find(|&i| {
                (type_bits & (1 << i)) != 0 &&
                    mem_props.memory_types[i].property_flags.contains(preferred)
            })
            .or_else(|| (0..count).find(|&i| (type_bits & (1 << i)) != 0))
            .map(|i| i as u32)
    }

    fn submit_commands(
        &self,
        record: impl FnOnce(&ash::Device, vk::CommandBuffer),
    ) -> Result<(), Error> {
        let state = self.submit.lock().unwrap();
        unsafe {
            self.device
                .reset_command_buffer(state.cmd_buf, vk::CommandBufferResetFlags::empty())
                .map_err(|e| Error::BackendSpecific(format!("reset_command_buffer: {e:?}")))?;
            self.device
                .begin_command_buffer(
                    state.cmd_buf,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|e| Error::BackendSpecific(format!("begin_command_buffer: {e:?}")))?;

            record(&self.device, state.cmd_buf);

            self.device
                .end_command_buffer(state.cmd_buf)
                .map_err(|e| Error::BackendSpecific(format!("end_command_buffer: {e:?}")))?;
            let cmd_bufs = [state.cmd_buf];
            let submit_info = [vk::SubmitInfo::default().command_buffers(&cmd_bufs)];
            self.device
                .queue_submit(self.queue, &submit_info, state.fence)
                .map_err(|e| Error::BackendSpecific(format!("queue_submit: {e:?}")))?;
            self.device
                .wait_for_fences(&[state.fence], true, u64::MAX)
                .map_err(|e| Error::BackendSpecific(format!("wait_for_fences: {e:?}")))?;
            self.device
                .reset_fences(&[state.fence])
                .map_err(|e| Error::BackendSpecific(format!("reset_fences: {e:?}")))?;
        }
        Ok(())
    }

    fn negotiate_drm_modifiers(
        &self,
        egl: &EglDmaBuf,
        display: EglDisplay,
        vk_format: vk::Format,
    ) -> Vec<u64> {
        if self.drm_format_modifier.is_none() {
            return Vec::new();
        }
        let Some(query_mods) = egl.query_dmabuf_modifiers else {
            return Vec::new();
        };
        let ext_ptr = unsafe { (egl.query_string)(display, EGL_EXTENSIONS) };
        if ext_ptr.is_null() {
            return Vec::new();
        }
        let exts = unsafe { CStr::from_ptr(ext_ptr) }.to_string_lossy();
        if !exts
            .split_whitespace()
            .any(|e| e == "EGL_EXT_image_dma_buf_import_modifiers")
        {
            return Vec::new();
        }

        let mut egl_count = 0i32;
        if unsafe {
            query_mods(
                display,
                DRM_FORMAT_ABGR8888 as i32,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut egl_count,
            )
        } != EGL_TRUE ||
            egl_count <= 0
        {
            return Vec::new();
        }
        let mut egl_mods = vec![0u64; egl_count as usize];
        let mut external_only = vec![0u32; egl_count as usize];
        if unsafe {
            query_mods(
                display,
                DRM_FORMAT_ABGR8888 as i32,
                egl_count,
                egl_mods.as_mut_ptr(),
                external_only.as_mut_ptr(),
                &mut egl_count,
            )
        } != EGL_TRUE
        {
            return Vec::new();
        }
        let renderable_egl_mods: Vec<u64> = egl_mods
            .into_iter()
            .zip(external_only)
            .take(egl_count as usize)
            .filter_map(|(m, ext_only)| (ext_only == 0).then_some(m))
            .collect();
        if renderable_egl_mods.is_empty() {
            return Vec::new();
        }

        let mut mod_list = vk::DrmFormatModifierPropertiesListEXT::default();
        let mut fmt_props = vk::FormatProperties2::default().push_next(&mut mod_list);
        unsafe {
            self.instance.get_physical_device_format_properties2(
                self.physical_device,
                vk_format,
                &mut fmt_props,
            );
        }
        if mod_list.drm_format_modifier_count == 0 {
            return Vec::new();
        }
        let mut vk_mods = vec![
            vk::DrmFormatModifierPropertiesEXT::default();
            mod_list.drm_format_modifier_count as usize
        ];
        let mut mod_list = vk::DrmFormatModifierPropertiesListEXT::default()
            .drm_format_modifier_properties(&mut vk_mods);
        let mut fmt_props = vk::FormatProperties2::default().push_next(&mut mod_list);
        unsafe {
            self.instance.get_physical_device_format_properties2(
                self.physical_device,
                vk_format,
                &mut fmt_props,
            );
        }

        let required = vk::FormatFeatureFlags::BLIT_SRC |
            vk::FormatFeatureFlags::TRANSFER_SRC |
            vk::FormatFeatureFlags::COLOR_ATTACHMENT;
        vk_mods
            .into_iter()
            .filter(|p| {
                p.drm_format_modifier_plane_count == 1 &&
                    p.drm_format_modifier_tiling_features.contains(required) &&
                    renderable_egl_mods.contains(&p.drm_format_modifier)
            })
            .map(|p| p.drm_format_modifier)
            .collect()
    }
}

pub(super) struct VulkanStagingImage {
    vulkan: Arc<VulkanContext>,
    image: vk::Image,
    memory: vk::DeviceMemory,
    egl_display: EglDisplay,
    egl_image: EglImage,
    gl_texture: gl::NativeTexture,
    size: Size2D<i32, UnknownUnit>,
}

impl Drop for VulkanStagingImage {
    fn drop(&mut self) {
        if let Ok(egl) = EGL_DMA_BUF.as_ref() &&
            !self.egl_image.is_null()
        {
            unsafe { (egl.destroy_image)(self.egl_display, self.egl_image) };
            self.egl_image = ptr::null_mut();
        }
        unsafe {
            self.vulkan.device.destroy_image(self.image, None);
            self.vulkan.device.free_memory(self.memory, None);
        }
    }
}

impl VulkanStagingImage {
    fn new(
        vulkan: Arc<VulkanContext>,
        gl: &gl::Context,
        size: Size2D<i32, UnknownUnit>,
        swapchain_format: u32,
    ) -> Result<Self, Error> {
        let egl = EGL_DMA_BUF
            .as_ref()
            .map_err(|e| Error::BackendSpecific(e.clone()))?;
        let display = unsafe { (egl.get_current_display)() };
        if display.is_null() {
            return Err(Error::BackendSpecific(
                "Current EGLDisplay is null while creating Vulkan WebXR layer".into(),
            ));
        }

        let vk_format = staging_vk_format(swapchain_format);
        let modifiers = vulkan.negotiate_drm_modifiers(egl, display, vk_format);
        if !modifiers.is_empty() {
            match Self::create_with_tiling(
                vulkan.clone(),
                egl,
                display,
                gl,
                size,
                vk_format,
                Some(&modifiers),
            ) {
                Ok(staging) => return Ok(staging),
                Err(err) => {
                    debug!(
                        "DMA-BUF modifier staging creation failed ({err:?}); falling back to LINEAR"
                    );
                },
            }
        }
        Self::create_with_tiling(vulkan, egl, display, gl, size, vk_format, None)
    }

    fn create_with_tiling(
        vulkan: Arc<VulkanContext>,
        egl: &EglDmaBuf,
        display: EglDisplay,
        gl: &gl::Context,
        size: Size2D<i32, UnknownUnit>,
        vk_format: vk::Format,
        modifiers: Option<&[u64]>,
    ) -> Result<Self, Error> {
        let mut ext_mem = vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let mut mod_list = vk::ImageDrmFormatModifierListCreateInfoEXT::default();
        let (tiling, usage) = if let Some(mods) = modifiers {
            mod_list = mod_list.drm_format_modifiers(mods);
            (
                vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT,
                vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            )
        } else {
            (vk::ImageTiling::LINEAR, vk::ImageUsageFlags::TRANSFER_SRC)
        };

        let mut img_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk_format)
            .extent(vk::Extent3D {
                width: size.width as u32,
                height: size.height as u32,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(tiling)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .push_next(&mut ext_mem);
        if modifiers.is_some() {
            img_info = img_info.push_next(&mut mod_list);
        }

        let image = unsafe { vulkan.device.create_image(&img_info, None) }
            .map_err(|e| Error::BackendSpecific(format!("vkCreateImage: {e:?}")))?;
        let mem_reqs = unsafe { vulkan.device.get_image_memory_requirements(image) };
        let Some(mem_type) = vulkan.find_memory_type(
            mem_reqs.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        ) else {
            unsafe { vulkan.device.destroy_image(image, None) };
            return Err(Error::BackendSpecific(
                "No suitable Vulkan memory type for DMA-BUF staging image".into(),
            ));
        };

        let mut export_info = vk::ExportMemoryAllocateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::default().image(image);
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_reqs.size)
            .memory_type_index(mem_type)
            .push_next(&mut export_info)
            .push_next(&mut dedicated_info);
        let memory = match unsafe { vulkan.device.allocate_memory(&alloc_info, None) } {
            Ok(m) => m,
            Err(e) => {
                unsafe { vulkan.device.destroy_image(image, None) };
                return Err(Error::BackendSpecific(format!("vkAllocateMemory: {e:?}")));
            },
        };

        if let Err(e) = unsafe { vulkan.device.bind_image_memory(image, memory, 0) } {
            unsafe {
                vulkan.device.destroy_image(image, None);
                vulkan.device.free_memory(memory, None);
            }
            return Err(Error::BackendSpecific(format!("vkBindImageMemory: {e:?}")));
        }

        // Transition UNDEFINED -> GENERAL once before GL imports and renders to the DMA-BUF.
        let subresource_range = color_subresource_range();
        if let Err(e) = vulkan.submit_commands(|device, cmd| {
            let barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::GENERAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(subresource_range);
            unsafe {
                device.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );
            }
        }) {
            unsafe {
                vulkan.device.destroy_image(image, None);
                vulkan.device.free_memory(memory, None);
            }
            return Err(e);
        }

        let (aspect, chosen_modifier) = if modifiers.is_some() {
            let mut mod_props = vk::ImageDrmFormatModifierPropertiesEXT::default();
            if let Err(e) = unsafe {
                vulkan
                    .drm_format_modifier
                    .as_ref()
                    .unwrap()
                    .get_image_drm_format_modifier_properties(image, &mut mod_props)
            } {
                unsafe {
                    vulkan.device.destroy_image(image, None);
                    vulkan.device.free_memory(memory, None);
                }
                return Err(Error::BackendSpecific(format!(
                    "vkGetImageDrmFormatModifierPropertiesEXT: {e:?}"
                )));
            }
            (
                vk::ImageAspectFlags::MEMORY_PLANE_0_EXT,
                Some(mod_props.drm_format_modifier),
            )
        } else {
            (vk::ImageAspectFlags::COLOR, None)
        };

        let layout = unsafe {
            vulkan.device.get_image_subresource_layout(
                image,
                vk::ImageSubresource {
                    aspect_mask: aspect,
                    mip_level: 0,
                    array_layer: 0,
                },
            )
        };
        let fd = match unsafe {
            vulkan.external_memory_fd.get_memory_fd(
                &vk::MemoryGetFdInfoKHR::default()
                    .memory(memory)
                    .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT),
            )
        } {
            Ok(fd) => fd,
            Err(e) => {
                unsafe {
                    vulkan.device.destroy_image(image, None);
                    vulkan.device.free_memory(memory, None);
                }
                return Err(Error::BackendSpecific(format!("vkGetMemoryFdKHR: {e:?}")));
            },
        };

        let mut attribs = vec![
            EGL_WIDTH,
            size.width,
            EGL_HEIGHT,
            size.height,
            EGL_LINUX_DRM_FOURCC_EXT,
            DRM_FORMAT_ABGR8888 as i32,
            EGL_DMA_BUF_PLANE0_FD_EXT,
            fd,
            EGL_DMA_BUF_PLANE0_OFFSET_EXT,
            layout.offset as i32,
            EGL_DMA_BUF_PLANE0_PITCH_EXT,
            layout.row_pitch as i32,
        ];
        if let Some(modifier) = chosen_modifier {
            attribs.extend_from_slice(&[
                EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT,
                (modifier & 0xFFFF_FFFF) as u32 as i32,
                EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT,
                (modifier >> 32) as u32 as i32,
            ]);
        }
        attribs.push(EGL_NONE);

        let egl_image = unsafe {
            (egl.create_image)(
                display,
                ptr::null_mut(),
                EGL_LINUX_DMA_BUF_EXT,
                ptr::null_mut(),
                attribs.as_ptr(),
            )
        };
        // EGL duplicates the DMA-BUF FD internally; always close our handle.
        unsafe { libc::close(fd) };

        if egl_image.is_null() {
            unsafe {
                vulkan.device.destroy_image(image, None);
                vulkan.device.free_memory(memory, None);
            }
            return Err(Error::BackendSpecific(
                "eglCreateImageKHR(EGL_LINUX_DMA_BUF_EXT) failed".into(),
            ));
        }

        let gl_texture = unsafe { bind_egl_image_to_gl_texture(egl, gl, egl_image) };
        let gl_texture = match gl_texture {
            Ok(tex) => tex,
            Err(err) => {
                unsafe {
                    (egl.destroy_image)(display, egl_image);
                    vulkan.device.destroy_image(image, None);
                    vulkan.device.free_memory(memory, None);
                }
                return Err(err);
            },
        };

        debug!(
            "Created Vulkan DMA-BUF staging layer ({}x{}, format={vk_format:?}, modifier={chosen_modifier:?})",
            size.width, size.height
        );
        Ok(Self {
            vulkan,
            image,
            memory,
            egl_display: display,
            egl_image,
            gl_texture,
            size,
        })
    }

    fn blit_to_swapchain_image(&self, swapchain_image: u64, gl: &gl::Context) -> Result<(), Error> {
        unsafe { gl.finish() };

        let dst_image = vk::Image::from_raw(swapchain_image);
        let subresource_range = color_subresource_range();
        let subresource_layers = vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        };
        let (width, height) = (self.size.width, self.size.height);

        self.vulkan.submit_commands(|device, cmd| {
            let pre_barriers = [
                vk::ImageMemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                    .old_layout(vk::ImageLayout::GENERAL)
                    .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(self.image)
                    .subresource_range(subresource_range),
                vk::ImageMemoryBarrier::default()
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(dst_image)
                    .subresource_range(subresource_range),
            ];
            // Hardware Y-flip from WebGL bottom-left origin to OpenXR Vulkan top-left origin.
            let blit = vk::ImageBlit {
                src_subresource: subresource_layers,
                src_offsets: [
                    vk::Offset3D {
                        x: 0,
                        y: height,
                        z: 0,
                    },
                    vk::Offset3D {
                        x: width,
                        y: 0,
                        z: 1,
                    },
                ],
                dst_subresource: subresource_layers,
                dst_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: width,
                        y: height,
                        z: 1,
                    },
                ],
            };
            let post_barriers = [
                vk::ImageMemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::TRANSFER_READ)
                    .dst_access_mask(vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE)
                    .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .new_layout(vk::ImageLayout::GENERAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(self.image)
                    .subresource_range(subresource_range),
                vk::ImageMemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(
                        vk::AccessFlags::COLOR_ATTACHMENT_READ |
                            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    )
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(dst_image)
                    .subresource_range(subresource_range),
            ];

            unsafe {
                device.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::ALL_COMMANDS,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &pre_barriers,
                );
                device.cmd_blit_image(
                    cmd,
                    self.image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    dst_image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[blit],
                    vk::Filter::NEAREST,
                );
                device.cmd_pipeline_barrier(
                    cmd,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::ALL_COMMANDS,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &post_barriers,
                );
            }
        })
    }

    pub(super) fn destroy(self, gl: Option<&gl::Context>) {
        if let Some(gl) = gl {
            unsafe { gl.delete_texture(self.gl_texture) };
        }
    }
}

unsafe fn bind_egl_image_to_gl_texture(
    egl: &EglDmaBuf,
    gl: &gl::Context,
    egl_image: EglImage,
) -> Result<gl::NativeTexture, Error> {
    unsafe {
        let prev_tex = gl.get_parameter_texture(gl::TEXTURE_BINDING_2D);
        let prev_draw_fbo = gl.get_parameter_framebuffer(gl::DRAW_FRAMEBUFFER_BINDING);
        let prev_read_fbo = gl.get_parameter_framebuffer(gl::READ_FRAMEBUFFER_BINDING);
        while gl.get_error() != gl::NO_ERROR {}

        let texture = gl
            .create_texture()
            .map_err(|e| Error::BackendSpecific(format!("glCreateTexture: {e}")))?;
        gl.bind_texture(gl::TEXTURE_2D, Some(texture));
        (egl.image_target_texture_2d)(gl::TEXTURE_2D, egl_image);
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        let tex_err = gl.get_error();

        let mut fbo_ok = false;
        if tex_err == gl::NO_ERROR &&
            let Ok(fbo) = gl.create_framebuffer()
        {
            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                Some(texture),
                0,
            );
            fbo_ok = gl.check_framebuffer_status(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE;
            gl.delete_framebuffer(fbo);
        }

        gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, prev_draw_fbo);
        gl.bind_framebuffer(gl::READ_FRAMEBUFFER, prev_read_fbo);
        gl.bind_texture(gl::TEXTURE_2D, prev_tex);

        if tex_err != gl::NO_ERROR || !fbo_ok {
            gl.delete_texture(texture);
            return Err(Error::BackendSpecific(format!(
                "DMA-BUF EGLImage texture invalid or FBO-incomplete (gl_err={tex_err:#x}, fbo_ok={fbo_ok})"
            )));
        }
        Ok(texture)
    }
}

fn color_subresource_range() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    }
}

fn staging_vk_format(swapchain_format: u32) -> vk::Format {
    match vk::Format::from_raw(swapchain_format as i32) {
        vk::Format::R8G8B8A8_SRGB | vk::Format::B8G8R8A8_SRGB => vk::Format::R8G8B8A8_SRGB,
        _ => vk::Format::R8G8B8A8_UNORM,
    }
}

impl GraphicsProviderMethods<Vulkan> for GraphicsProvider {
    fn needs_y_flip() -> bool {
        // `blit_to_swapchain_image` performs the Y-flip in hardware during presentation.
        false
    }

    fn swapchain_usage_flags() -> SwapchainUsageFlags {
        SwapchainUsageFlags::COLOR_ATTACHMENT |
            SwapchainUsageFlags::SAMPLED |
            SwapchainUsageFlags::TRANSFER_DST
    }

    fn enable_graphics_extensions_for(
        supported: &ExtensionSet,
        exts: &mut ExtensionSet,
    ) -> Result<(), String> {
        if !supported.khr_vulkan_enable2 {
            return Err("OpenXR runtime does not support XR_KHR_vulkan_enable2".into());
        }
        exts.khr_vulkan_enable2 = true;
        Ok(())
    }

    fn try_pick_format(formats: &[u32]) -> Result<u32, Error> {
        debug!("Available OpenXR Vulkan swapchain formats: {formats:?}");
        [
            vk::Format::R8G8B8A8_SRGB.as_raw() as u32,
            vk::Format::B8G8R8A8_SRGB.as_raw() as u32,
            vk::Format::R8G8B8A8_UNORM.as_raw() as u32,
            vk::Format::B8G8R8A8_UNORM.as_raw() as u32,
        ]
        .into_iter()
        .find(|preferred| formats.contains(preferred))
        .or_else(|| formats.first().copied())
        .ok_or_else(|| {
            Error::BackendSpecific("OpenXR runtime returned no swapchain formats".into())
        })
    }

    fn create_session_with_graphics(
        instance: &Instance,
        system: SystemId,
        _device: &SurfmanDevice,
    ) -> Result<
        (
            Session<Vulkan>,
            FrameWaiter,
            FrameStream<Vulkan>,
            SessionGraphics,
        ),
        Error,
    > {
        let vulkan = VulkanContext::new(instance, system)?;
        debug!(
            "Creating OpenXR Vulkan session (queue_family={}, drm_modifiers={})",
            vulkan.queue_family_index,
            vulkan.drm_format_modifier.is_some()
        );
        let info = openxr::vulkan::SessionCreateInfo {
            instance: vulkan.instance.handle().as_raw() as _,
            physical_device: vulkan.physical_device.as_raw() as _,
            device: vulkan.device.handle().as_raw() as _,
            queue_family_index: vulkan.queue_family_index,
            queue_index: 0,
        };
        // Keep VkDevice/VkInstance alive until after xrDestroySession.
        let (session, waiter, stream) = unsafe {
            instance
                .create_session_with_guard::<Vulkan>(system, &info, Box::new(vulkan.clone()))
                .map_err(|e| Error::BackendSpecific(format!("Instance::create_session {:?}", e)))?
        };
        Ok((
            session,
            waiter,
            stream,
            SessionGraphics {
                vulkan: Some(vulkan),
            },
        ))
    }

    fn create_layer_graphics(
        session_graphics: &SessionGraphics,
        gl: &gl::Context,
        size: &Size2D<i32, UnknownUnit>,
        format: u32,
    ) -> Result<LayerGraphics, Error> {
        let vulkan = session_graphics
            .vulkan
            .as_ref()
            .ok_or(Error::NoMatchingDevice)?
            .clone();
        let staging = VulkanStagingImage::new(vulkan, gl, *size, format)?;
        Ok(LayerGraphics {
            vulkan: Some(staging),
        })
    }

    fn create_swapchain_texture(
        _image: u64,
        _device: &SurfmanDevice,
        _context: &mut SurfmanContext,
        _size: &Size2D<i32, UnknownUnit>,
        layer_graphics: &LayerGraphics,
    ) -> Result<SwapchainTexture, SurfmanError> {
        layer_graphics
            .vulkan
            .as_ref()
            .map(|staging| SwapchainTexture::GlTexture(staging.gl_texture))
            .ok_or(SurfmanError::Failed)
    }

    fn present_layer(
        layer_graphics: &LayerGraphics,
        image: u64,
        gl: &gl::Context,
    ) -> Result<(), Error> {
        if let Some(staging) = layer_graphics.vulkan.as_ref() {
            staging.blit_to_swapchain_image(image, gl)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use surfman::{
        Connection as SurfmanConnection, ContextAttributeFlags, ContextAttributes, GLApi, GLVersion,
    };

    use super::*;

    #[test]
    fn format_selection_and_flags() {
        let rgba_srgb = vk::Format::R8G8B8A8_SRGB.as_raw() as u32;
        let bgra_srgb = vk::Format::B8G8R8A8_SRGB.as_raw() as u32;
        let rgba_unorm = vk::Format::R8G8B8A8_UNORM.as_raw() as u32;
        let bgra_unorm = vk::Format::B8G8R8A8_UNORM.as_raw() as u32;
        let other = vk::Format::R16G16B16A16_SFLOAT.as_raw() as u32;

        assert_eq!(
            GraphicsProvider::try_pick_format(&[
                other, bgra_unorm, rgba_unorm, bgra_srgb, rgba_srgb
            ])
            .unwrap(),
            rgba_srgb
        );
        assert_eq!(
            GraphicsProvider::try_pick_format(&[other, rgba_unorm, bgra_srgb]).unwrap(),
            bgra_srgb
        );
        assert_eq!(
            GraphicsProvider::try_pick_format(&[other, bgra_unorm, rgba_unorm]).unwrap(),
            rgba_unorm
        );
        assert_eq!(
            GraphicsProvider::try_pick_format(&[other, bgra_unorm]).unwrap(),
            bgra_unorm
        );
        assert_eq!(GraphicsProvider::try_pick_format(&[other]).unwrap(), other);
        assert!(GraphicsProvider::try_pick_format(&[]).is_err());

        assert_eq!(staging_vk_format(rgba_srgb), vk::Format::R8G8B8A8_SRGB);
        assert_eq!(staging_vk_format(bgra_srgb), vk::Format::R8G8B8A8_SRGB);
        assert_eq!(staging_vk_format(rgba_unorm), vk::Format::R8G8B8A8_UNORM);
        assert_eq!(staging_vk_format(other), vk::Format::R8G8B8A8_UNORM);

        assert!(!<GraphicsProvider as GraphicsProviderMethods<Vulkan>>::needs_y_flip());
        let usage = <GraphicsProvider as GraphicsProviderMethods<Vulkan>>::swapchain_usage_flags();
        assert!(usage.contains(SwapchainUsageFlags::COLOR_ATTACHMENT));
        assert!(usage.contains(SwapchainUsageFlags::TRANSFER_DST));
    }

    #[test]
    fn enable_graphics_extensions_requires_vulkan_enable2() {
        let mut supported = ExtensionSet::default();
        let mut exts = ExtensionSet::default();
        assert!(GraphicsProvider::enable_graphics_extensions_for(&supported, &mut exts).is_err());

        supported.khr_vulkan_enable2 = true;
        assert!(GraphicsProvider::enable_graphics_extensions_for(&supported, &mut exts).is_ok());
        assert!(exts.khr_vulkan_enable2);
    }

    #[test]
    fn vulkan_dmabuf_egl_interop_and_y_flip_blit() {
        let Ok(connection) = SurfmanConnection::new() else {
            return;
        };
        let Ok(adapter) = connection.create_adapter() else {
            return;
        };
        let Ok(device) = connection.create_device(&adapter) else {
            return;
        };
        let version = match device.gl_api() {
            GLApi::GL => GLVersion::new(3, 2),
            GLApi::GLES => GLVersion::new(3, 0),
        };
        let Ok(desc) = device.create_context_descriptor(&ContextAttributes {
            version,
            flags: ContextAttributeFlags::ALPHA | ContextAttributeFlags::DEPTH,
        }) else {
            return;
        };
        let Ok(mut ctx) = device.create_context(&desc, None) else {
            return;
        };
        device.make_context_current(&ctx).unwrap();
        let gl = unsafe {
            gl::Context::from_loader_function(|name| device.get_proc_address(&ctx, name))
        };

        let Ok(entry) = (unsafe { ash::Entry::load() }) else {
            device.destroy_context(&mut ctx).unwrap();
            return;
        };
        let app_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_1);
        let Ok(instance) = (unsafe {
            entry.create_instance(
                &vk::InstanceCreateInfo::default().application_info(&app_info),
                None,
            )
        }) else {
            device.destroy_context(&mut ctx).unwrap();
            return;
        };

        let phys_devices = unsafe { instance.enumerate_physical_devices() }.unwrap_or_default();
        let Some((physical_device, queue_family_index, enable_drm_mod)) =
            phys_devices.into_iter().find_map(|pd| {
                let props = unsafe { instance.get_physical_device_properties(pd) };
                if props.api_version < vk::API_VERSION_1_1 {
                    return None;
                }
                let qfi = unsafe { instance.get_physical_device_queue_family_properties(pd) }
                    .into_iter()
                    .enumerate()
                    .find_map(|(i, q)| {
                        q.queue_flags
                            .contains(vk::QueueFlags::GRAPHICS)
                            .then_some(i as u32)
                    })?;
                let exts = unsafe { instance.enumerate_device_extension_properties(pd) }.ok()?;
                let has = |n: &CStr| {
                    exts.iter()
                        .any(|p| unsafe { CStr::from_ptr(p.extension_name.as_ptr()) } == n)
                };
                if !has(ash::khr::external_memory_fd::NAME) ||
                    !has(ash::ext::external_memory_dma_buf::NAME)
                {
                    return None;
                }
                let drm_mod = has(ash::ext::image_drm_format_modifier::NAME) &&
                    has(ash::khr::image_format_list::NAME);
                Some((pd, qfi, drm_mod))
            })
        else {
            unsafe { instance.destroy_instance(None) };
            device.destroy_context(&mut ctx).unwrap();
            return;
        };

        let mut ext_ptrs = vec![
            ash::khr::external_memory_fd::NAME.as_ptr(),
            ash::ext::external_memory_dma_buf::NAME.as_ptr(),
        ];
        if enable_drm_mod {
            ext_ptrs.push(ash::ext::image_drm_format_modifier::NAME.as_ptr());
            ext_ptrs.push(ash::khr::image_format_list::NAME.as_ptr());
        }
        let priorities = [1.0f32];
        let queue_infos = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)];
        let vk_device = unsafe {
            instance.create_device(
                physical_device,
                &vk::DeviceCreateInfo::default()
                    .queue_create_infos(&queue_infos)
                    .enabled_extension_names(&ext_ptrs),
                None,
            )
        }
        .unwrap();
        let vulkan = VulkanContext::from_handles(
            entry,
            instance,
            physical_device,
            vk_device,
            queue_family_index,
            enable_drm_mod,
        )
        .unwrap();

        let size = Size2D::new(16, 16);
        let fmt = vk::Format::R8G8B8A8_UNORM.as_raw() as u32;
        let staging = VulkanStagingImage::new(vulkan.clone(), &gl, size, fmt).unwrap();

        // Render RED to top half (GL y=8..16) and GREEN to bottom half (GL y=0..8).
        unsafe {
            let fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                Some(staging.gl_texture),
                0,
            );
            assert_eq!(
                gl.check_framebuffer_status(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE
            );
            gl.enable(gl::SCISSOR_TEST);
            gl.scissor(0, 8, 16, 8);
            gl.clear_color(1.0, 0.0, 0.0, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT);
            gl.scissor(0, 0, 16, 8);
            gl.clear_color(0.0, 1.0, 0.0, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT);
            gl.disable(gl::SCISSOR_TEST);
            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            gl.delete_framebuffer(fbo);
        }

        // Create a linear host-visible Vulkan destination image to verify Y-flipped blit output.
        let dst_img = unsafe {
            vulkan.device.create_image(
                &vk::ImageCreateInfo::default()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(vk::Format::R8G8B8A8_UNORM)
                    .extent(vk::Extent3D {
                        width: 16,
                        height: 16,
                        depth: 1,
                    })
                    .mip_levels(1)
                    .array_layers(1)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .tiling(vk::ImageTiling::LINEAR)
                    .usage(vk::ImageUsageFlags::TRANSFER_DST)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .initial_layout(vk::ImageLayout::UNDEFINED),
                None,
            )
        }
        .unwrap();
        let reqs = unsafe { vulkan.device.get_image_memory_requirements(dst_img) };
        let mem_type = vulkan
            .find_memory_type(
                reqs.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .unwrap();
        let dst_mem = unsafe {
            vulkan.device.allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(reqs.size)
                    .memory_type_index(mem_type),
                None,
            )
        }
        .unwrap();
        unsafe { vulkan.device.bind_image_memory(dst_img, dst_mem, 0) }.unwrap();

        staging
            .blit_to_swapchain_image(dst_img.as_raw(), &gl)
            .unwrap();

        let layout = unsafe {
            vulkan.device.get_image_subresource_layout(
                dst_img,
                vk::ImageSubresource {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    array_layer: 0,
                },
            )
        };
        let mapped = unsafe {
            vulkan
                .device
                .map_memory(dst_mem, 0, reqs.size, vk::MemoryMapFlags::empty())
        }
        .unwrap() as *const u8;

        // In Vulkan top-left coordinates: row 2 should be RED, row 13 should be GREEN.
        let pixel_at = |x: usize, y: usize| -> [u8; 4] {
            let off = layout.offset as usize + y * layout.row_pitch as usize + x * 4;
            unsafe {
                [
                    *mapped.add(off),
                    *mapped.add(off + 1),
                    *mapped.add(off + 2),
                    *mapped.add(off + 3),
                ]
            }
        };
        assert_eq!(pixel_at(8, 2), [255, 0, 0, 255], "top row should be RED");
        assert_eq!(
            pixel_at(8, 13),
            [0, 255, 0, 255],
            "bottom row should be GREEN"
        );

        unsafe {
            vulkan.device.unmap_memory(dst_mem);
            vulkan.device.destroy_image(dst_img, None);
            vulkan.device.free_memory(dst_mem, None);
        }
        staging.destroy(Some(&gl));
        device.destroy_context(&mut ctx).unwrap();
    }
}
