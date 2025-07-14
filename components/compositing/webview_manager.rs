/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::panic;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::hash_map::{Values, ValuesMut};
use std::rc::Rc;

use base::id::{RenderingGroupId, WebViewId};
use compositing_traits::rendering_context::RenderingContext;
use compositing_traits::{CompositorMsg, CompositorProxy};
use euclid::Size2D;
use gleam::gl::Gl;
use log::{error, info};
use servo_config::pref;
use webrender::{
    RenderApi, RenderApiSender, ShaderPrecacheFlags, Transaction, UploadMethod, VertexUsageHint,
    WebRenderOptions,
};
use webrender_api::units::DevicePixel;
use webrender_api::{
    ColorF, DocumentId, FramePublishId, FrameReadyParams, IdNamespace, RenderNotifier,
};

use crate::compositor::RepaintReason;
use crate::webview_renderer::UnknownWebView;

pub(crate) struct WebRenderInstance {
    pub(crate) rendering_context: Rc<dyn RenderingContext>,
    pub(crate) webrender: webrender::Renderer,
    pub(crate) webrender_gl: Rc<dyn Gl>,
    pub(crate) webrender_document: DocumentId,
    pub(crate) webrender_api: RenderApi,
    pub(crate) needs_repaint: Cell<RepaintReason>,
    sender: RenderApiSender,
    notifier: MyRenderNotifier,
}

struct MyRenderNotifier {
    frame_ready_msg: RefCell<Vec<(DocumentId, bool)>>,
    sender: CompositorProxy,
    group_id: RenderingGroupId,
}

impl MyRenderNotifier {
    pub fn new(sender: CompositorProxy, group_id: RenderingGroupId) -> MyRenderNotifier {
        MyRenderNotifier {
            frame_ready_msg: RefCell::new(vec![]),
            sender,
            group_id,
        }
    }
}

impl webrender_api::RenderNotifier for MyRenderNotifier {
    fn clone(&self) -> Box<dyn webrender_api::RenderNotifier> {
        Box::new(MyRenderNotifier {
            frame_ready_msg: self.frame_ready_msg.clone(),
            sender: self.sender.clone(),
            group_id: self.group_id.clone(),
        })
    }

    fn wake_up(&self, _composite_needed: bool) {}

    fn new_frame_ready(
        &self,
        document_id: DocumentId,
        _: FramePublishId,
        frame_ready_params: &FrameReadyParams,
    ) {
        self.sender.send(CompositorMsg::NewWebRenderFrameReady(
            document_id,
            self.group_id.clone(),
            frame_ready_params.render,
        ));
        log::info!("RenderNotifier push from {document_id:?}");
    }
}

pub(crate) struct WebViewManager<WebView> {
    /// Our top-level browsing contexts. In the WebRender scene, their pipelines are the children of
    /// a single root pipeline that also applies any pinch zoom transformation.
    webviews: HashMap<WebViewId, WebView>,

    rendering_contexts: HashMap<RenderingGroupId, WebRenderInstance>,

    webview_groups: HashMap<WebViewId, RenderingGroupId>,

    /// The order to paint them in, topmost last.
    painting_order: HashMap<RenderingGroupId, Vec<WebViewId>>,

    last_used_id: Option<RenderingGroupId>,

    sender: CompositorProxy,
}

impl<WebView> WebViewManager<WebView> {
    pub(crate) fn new(sender: CompositorProxy) -> Self {
        Self {
            webviews: Default::default(),
            painting_order: Default::default(),
            webview_groups: Default::default(),
            rendering_contexts: Default::default(),
            last_used_id: None,
            sender,
        }
    }
}

fn gl_error_panic(_gl: &dyn Gl, s: &str, e: gleam::gl::GLenum) {
    panic!("FOUND GL ERROR s: {:?}, e: {:?}", s, e);
}

impl<WebView> WebViewManager<WebView> {
    pub(crate) fn rendering_contexts(&self) -> impl Iterator<Item = &WebRenderInstance> {
        self.rendering_contexts.iter().map(|(_, v)| v)
    }

    pub(crate) fn clear_background(&self, webview_group_id: RenderingGroupId) {
        let rtc = self.rendering_contexts.get(&webview_group_id).unwrap();
        rtc.rendering_context.make_current().expect("Make current");
        let gl = &rtc.webrender_gl;

        {
            debug_assert_eq!(
                (
                    gl.get_error(),
                    gl.check_frame_buffer_status(gleam::gl::FRAMEBUFFER)
                ),
                (gleam::gl::NO_ERROR, gleam::gl::FRAMEBUFFER_COMPLETE)
            );
        }

        // Always clear the entire RenderingContext, regardless of how many WebViews there are
        // or where they are positioned. This is so WebView actually clears even before the
        // first WebView is ready.
        let color = servo_config::pref!(shell_background_color_rgba);

        gl.clear_color(
            color[0] as f32,
            color[1] as f32,
            color[2] as f32,
            color[3] as f32,
        );
        gl.clear(gleam::gl::COLOR_BUFFER_BIT);
    }

    pub(crate) fn needs_repaint(&self) -> RepaintReason {
        let mut repaint_reason = RepaintReason::default();
        for i in self.rendering_contexts.values() {
            repaint_reason = i.needs_repaint.get().union(repaint_reason);
        }
        repaint_reason
    }

    pub(crate) fn send_transaction(&mut self, webview_id: WebViewId, transaction: Transaction) {
        let gid = self.group_id(webview_id).unwrap();
        self.send_transaction_to_group(gid, transaction);
    }

    pub(crate) fn assert_no_gl_error(&self, group_id: RenderingGroupId) {
        let rtc = self
            .rendering_contexts
            .get(&group_id)
            .expect(&format!("No group {:?}", group_id));
        debug_assert_eq!(rtc.webrender_gl.get_error(), gleam::gl::NO_ERROR);
    }

    pub(crate) fn send_transaction_to_group(
        &mut self,
        gid: RenderingGroupId,
        transaction: Transaction,
    ) {
        self.assert_no_gl_error(gid.clone());
        let rect = self.rendering_contexts.get_mut(&gid).unwrap();
        rect.webrender_api
            .send_transaction(rect.webrender_document, transaction);
    }

    // This sends the transaction to the namespace id. It requires that the namespace id is unique
    // and no multiple webrender instances are using it. Panics if the given id has multiple groups with
    // the same id.
    pub(crate) fn send_transaction_to_namespace_id(
        &mut self,
        transaction: Transaction,
        id: IdNamespace,
    ) {
        let mut namespace_ids = self
            .rendering_contexts
            .iter()
            .filter(|(_group, instance)| instance.webrender_api.get_namespace_id() == id);
        assert_eq!(namespace_ids.clone().count(), 1);

        if let Some((group, _instance)) = namespace_ids.next() {
            self.send_transaction_to_group(group.clone(), transaction);
        } else {
            error!("Could not find namespace, something is wrong");
        }
    }

    // Flush scene builder for all rendering groups
    pub(crate) fn flush_scene_builder(&self) {
        for (key, i) in self.rendering_contexts.iter() {
            self.assert_no_gl_error(key.clone());
            i.webrender_api.flush_scene_builder();
        }
    }

    // Deinit all groups
    pub(crate) fn deinit(&mut self) {
        for (_group_id, webrender_instance) in self.rendering_contexts.drain() {
            webrender_instance
                .rendering_context
                .make_current()
                .expect("Foo");
            webrender_instance.webrender.deinit();
        }
        self.last_used_id = None;
        self.painting_order.clear();
        self.webviews.clear();
        self.webview_groups.clear();
    }

    fn group_painting_order_mut(&mut self, webview_id: WebViewId) -> &mut Vec<WebViewId> {
        let group_id = self.webview_groups.get(&webview_id).unwrap();
        self.painting_order.get_mut(group_id).unwrap()
    }

    pub(crate) fn webrender_instance(&self, group_id: RenderingGroupId) -> &WebRenderInstance {
        self.assert_no_gl_error(group_id.clone());
        self.rendering_contexts.get(&group_id).unwrap()
    }

    // Gets the `DocumentId` for a `WebViewId`
    #[allow(unused)]
    pub(crate) fn document_id(&self, webview_id: &WebViewId) -> DocumentId {
        self.webview_groups
            .get(webview_id)
            .and_then(|rgid| {
                self.assert_no_gl_error(rgid.clone());
                self.rendering_contexts.get(rgid)
            })
            .map(|rg| rg.webrender_document)
            .expect("Could not find")
    }

    pub(crate) fn render_instance(&self, group_id: RenderingGroupId) -> &WebRenderInstance {
        if let Some(wr) = self.rendering_contexts.get(&group_id) {
            wr
        } else {
            panic!("Could not find WebrenderInstance with group id {group_id:?}");
        }
    }

    pub(crate) fn render_instance_mut(
        &mut self,
        group_id: RenderingGroupId,
    ) -> &mut WebRenderInstance {
        self.assert_no_gl_error(group_id.clone());
        if let Some(wr) = self.rendering_contexts.get_mut(&group_id) {
            wr
        } else {
            panic!("Could not find WebRenderInstance with group id {group_id:?}");
        }
    }

    fn webrender_options(&self, id: RenderingGroupId) -> WebRenderOptions {
        let clear_color = ColorF::new(1.0, 1.0, 1.0, 1.0);
        webrender::WebRenderOptions {
            // We force the use of optimized shaders here because rendering is broken
            // on Android emulators with unoptimized shaders. This is due to a known
            // issue in the emulator's OpenGL emulation layer.
            // See: https://github.com/servo/servo/issues/31726
            use_optimized_shaders: false,
            //resource_override_path: opts.shaders_dir.clone(),
            precache_flags: if pref!(gfx_precache_shaders) {
                ShaderPrecacheFlags::FULL_COMPILE
            } else {
                ShaderPrecacheFlags::empty()
            },
            enable_aa: pref!(gfx_text_antialiasing_enabled),
            enable_subpixel_aa: pref!(gfx_subpixel_text_antialiasing_enabled),
            allow_texture_swizzling: pref!(gfx_texture_swizzling_enabled),
            clear_color,
            upload_method: UploadMethod::PixelBuffer(VertexUsageHint::Stream),
            panic_on_gl_error: true,
            size_of_op: Some(servo_allocator::usable_size),
            renderer_id: id.render_id(),
            ..Default::default()
        }
    }

    pub(crate) fn add_webview_group(
        &mut self,
        new_group_id: Option<RenderingGroupId>,
        rendering_context: Rc<dyn RenderingContext>,
    ) -> RenderingGroupId {
        let new_group_id = new_group_id.unwrap_or_default();

        //let gl = gleam::gl::ErrorReactingGl::wrap(rendering_context.gleam_gl_api(), gl_error_panic);
        let gl = rendering_context.gleam_gl_api();

        debug_assert_eq!(
            (
                gl.get_error(),
                gl.check_frame_buffer_status(gleam::gl::FRAMEBUFFER)
            ),
            (gleam::gl::NO_ERROR, gleam::gl::FRAMEBUFFER_COMPLETE)
        );
        let notifier = MyRenderNotifier::new(self.sender.clone(), new_group_id.clone());

        let (webrender, sender) = webrender::create_webrender_instance(
            gl.clone(),
            notifier.clone(),
            self.webrender_options(new_group_id.clone()),
            None,
        )
        .expect("Could not");

        let api = sender.create_api();
        let webrender_document = api.add_document(rendering_context.size2d().to_i32());

        let s = WebRenderInstance {
            sender,
            webrender_api: api,
            webrender_document,
            rendering_context,
            webrender,
            webrender_gl: gl,
            notifier,
            needs_repaint: Cell::default(),
        };

        // This would otherwise drop the previous webrender instance which will error
        // in mysterious ways
        assert!(!self.rendering_contexts.contains_key(&new_group_id));

        self.rendering_contexts.insert(new_group_id.clone(), s);
        self.painting_order.insert(new_group_id.clone(), vec![]);
        self.assert_no_gl_error(new_group_id.clone());
        new_group_id
    }

    pub(crate) fn set_webrender_debug_flags(&mut self, flags: webrender_api::DebugFlags) {
        for webrender in self
            .rendering_contexts
            .values_mut()
            .map(|rc| &mut rc.webrender)
        {
            webrender.set_debug_flags(flags);
        }
    }

    pub(crate) fn groups(&self) -> Vec<RenderingGroupId> {
        self.painting_order.keys().cloned().collect()
    }

    pub(crate) fn rendering_context_size(&self) -> Size2D<u32, DevicePixel> {
        self.rendering_contexts
            .values()
            .next()
            .expect("No Context")
            .rendering_context
            .size2d()
    }

    pub(crate) fn present_all(&self) {
        for webrender in self.rendering_contexts() {
            webrender.rendering_context.present();
        }
    }

    pub(crate) fn group_id(&self, webview_id: WebViewId) -> Option<RenderingGroupId> {
        self.webview_groups.get(&webview_id).cloned()
    }

    pub(crate) fn remove(&mut self, webview_id: WebViewId) -> Result<WebView, UnknownWebView> {
        let painting_order = self.group_painting_order_mut(webview_id);
        painting_order.retain(|b| *b != webview_id);
        self.webviews
            .remove(&webview_id)
            .ok_or(UnknownWebView(webview_id))
    }

    pub(crate) fn get_webview(&self, webview_id: WebViewId) -> Option<&WebView> {
        self.webviews.get(&webview_id)
    }

    pub(crate) fn get_webview_webrender_mut(
        &mut self,
        webview_id: WebViewId,
    ) -> Option<(&mut WebView, &mut WebRenderInstance)> {
        let group_id = self
            .group_id(webview_id)
            .expect("Could not find webrender instance")
            .clone();
        //let wri = self.render_instance(group_id);
        let wri = self.rendering_contexts.get_mut(&group_id).unwrap();
        self.webviews.get_mut(&webview_id).map(|wv| (wv, wri))
    }

    pub(crate) fn get_webview_mut(&mut self, webview_id: WebViewId) -> Option<&mut WebView> {
        self.webviews.get_mut(&webview_id)
    }

    /// Returns true iff the painting order actually changed.
    pub(crate) fn show(&mut self, webview_id: WebViewId) -> Result<bool, UnknownWebView> {
        if !self.webviews.contains_key(&webview_id) {
            return Err(UnknownWebView(webview_id));
        }
        let painting_order = self.group_painting_order_mut(webview_id);
        if !painting_order.contains(&webview_id) {
            painting_order.push(webview_id);
            return Ok(true);
        }
        Ok(false)
    }

    /// Returns true iff the painting order actually changed.
    pub(crate) fn hide(&mut self, webview_id: WebViewId) -> Result<bool, UnknownWebView> {
        if !self.webviews.contains_key(&webview_id) {
            return Err(UnknownWebView(webview_id));
        }
        let painting_order = self.group_painting_order_mut(webview_id);
        if painting_order.contains(&webview_id) {
            painting_order.retain(|b| *b != webview_id);
            return Ok(true);
        }
        Ok(false)
    }

    /// Returns true iff the painting order actually changed.
    pub(crate) fn hide_all(&mut self, group_id: RenderingGroupId) -> bool {
        let v = self.painting_order.get_mut(&group_id);
        let painting_order = v.unwrap();
        if !painting_order.is_empty() {
            painting_order.clear();
            return true;
        }
        false
    }

    /// Returns true iff the painting order actually changed.
    pub(crate) fn raise_to_top(&mut self, webview_id: WebViewId) -> Result<bool, UnknownWebView> {
        if !self.webviews.contains_key(&webview_id) {
            return Err(UnknownWebView(webview_id));
        }
        let painting_order = self.group_painting_order_mut(webview_id);
        if painting_order.last() != Some(&webview_id) {
            self.hide(webview_id)?;
            self.show(webview_id)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(crate) fn painting_order(
        &self,
        group_id: RenderingGroupId,
    ) -> impl Iterator<Item = (&WebViewId, &WebView)> {
        info!(
            "groups: {:?} || wvs: {:?} || groupid {:?} || painting {:?}",
            self.webview_groups,
            self.webviews.keys(),
            group_id,
            self.painting_order
        );
        self.painting_order
            .get(&group_id)
            .expect("Could not find group")
            .iter()
            .flat_map(move |webview_id| self.get_webview(*webview_id).map(|b| (webview_id, b)))
    }

    pub(crate) fn add_webview(
        &mut self,
        group_id: RenderingGroupId,
        webview_id: WebViewId,
        webview: WebView,
    ) {
        self.webviews.entry(webview_id).or_insert(webview);
        self.webview_groups.entry(webview_id).or_insert(group_id);
    }

    pub(crate) fn iter(&self) -> Values<'_, WebViewId, WebView> {
        self.webviews.values()
    }

    /// Mutable iterator for `WebView` and `WebRenderInstance`
    pub(crate) fn webrender_instance_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut WebView, &WebRenderInstance)> {
        self.webviews.iter_mut().map(|(id, wv)| {
            (
                wv,
                self.webview_groups
                    .get(id)
                    .and_then(|gid| self.rendering_contexts.get(gid))
                    .expect("Could not get gid"),
            )
        })
    }

    pub(crate) fn iter_mut(&mut self) -> ValuesMut<'_, WebViewId, WebView> {
        self.webviews.values_mut()
    }
}

#[cfg(test)]
mod test {
    use base::id::{BrowsingContextId, Index, PipelineNamespace, PipelineNamespaceId, WebViewId};

    use crate::webview_manager::WebViewManager;
    use crate::webview_renderer::UnknownWebView;

    fn top_level_id(namespace_id: u32, index: u32) -> WebViewId {
        WebViewId(BrowsingContextId {
            namespace_id: PipelineNamespaceId(namespace_id),
            index: Index::new(index).unwrap(),
        })
    }

    fn webviews_sorted<WebView: Clone>(
        webviews: &WebViewManager<WebView>,
    ) -> Vec<(WebViewId, WebView)> {
        let mut keys = webviews.webviews.keys().collect::<Vec<_>>();
        keys.sort_unstable();
        keys.iter()
            .map(|&id| (*id, webviews.webviews.get(id).cloned().unwrap()))
            .collect()
    }

    #[test]
    fn test() {
        PipelineNamespace::install(PipelineNamespaceId(0));
        let mut webviews = WebViewManager::default();

        // entry() adds the webview to the map, but not the painting order.
        webviews.entry(WebViewId::new()).or_insert('a');
        webviews.entry(WebViewId::new()).or_insert('b');
        webviews.entry(WebViewId::new()).or_insert('c');
        assert!(webviews.get(top_level_id(0, 1)).is_some());
        assert!(webviews.get(top_level_id(0, 2)).is_some());
        assert!(webviews.get(top_level_id(0, 3)).is_some());
        assert_eq!(
            webviews_sorted(&webviews),
            vec![
                (top_level_id(0, 1), 'a'),
                (top_level_id(0, 2), 'b'),
                (top_level_id(0, 3), 'c'),
            ]
        );
        assert!(webviews.painting_order.is_empty());

        // add() returns WebViewAlreadyExists if the webview id already exists.
        webviews.entry(top_level_id(0, 3)).or_insert('d');
        assert!(webviews.get_webview(top_level_id(0, 3)).is_some());

        // Other methods return UnknownWebView or None if the webview id doesn’t exist.
        assert_eq!(
            webviews.remove(top_level_id(1, 1)),
            Err(UnknownWebView(top_level_id(1, 1)))
        );
        assert_eq!(webviews.get_webview(top_level_id(1, 1)), None);
        assert_eq!(webviews.get_webview_mut(top_level_id(1, 1)), None);
        assert_eq!(
            webviews.show(top_level_id(1, 1)),
            Err(UnknownWebView(top_level_id(1, 1)))
        );
        assert_eq!(
            webviews.hide(top_level_id(1, 1)),
            Err(UnknownWebView(top_level_id(1, 1)))
        );
        assert_eq!(
            webviews.raise_to_top(top_level_id(1, 1)),
            Err(UnknownWebView(top_level_id(1, 1)))
        );

        // For webviews not yet visible, both show() and raise_to_top() add the given webview on top.
        assert_eq!(webviews.show(top_level_id(0, 2)), Ok(true));
        assert_eq!(webviews.show(top_level_id(0, 2)), Ok(false));
        assert_eq!(webviews.painting_order, vec![top_level_id(0, 2)]);
        assert_eq!(webviews.raise_to_top(top_level_id(0, 1)), Ok(true));
        assert_eq!(webviews.raise_to_top(top_level_id(0, 1)), Ok(false));
        assert_eq!(
            webviews.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1)]
        );
        assert_eq!(webviews.show(top_level_id(0, 3)), Ok(true));
        assert_eq!(webviews.show(top_level_id(0, 3)), Ok(false));
        assert_eq!(
            webviews.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1), top_level_id(0, 3)]
        );

        // For webviews already visible, show() does nothing, while raise_to_top() makes it on top.
        assert_eq!(webviews.show(top_level_id(0, 1)), Ok(false));
        assert_eq!(
            webviews.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1), top_level_id(0, 3)]
        );
        assert_eq!(webviews.raise_to_top(top_level_id(0, 1)), Ok(true));
        assert_eq!(webviews.raise_to_top(top_level_id(0, 1)), Ok(false));
        assert_eq!(
            webviews.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 3), top_level_id(0, 1)]
        );

        // hide() removes the webview from the painting order, but not the map.
        assert_eq!(webviews.hide(top_level_id(0, 3)), Ok(true));
        assert_eq!(webviews.hide(top_level_id(0, 3)), Ok(false));
        assert_eq!(
            webviews.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1)]
        );
        assert_eq!(
            webviews_sorted(&webviews),
            vec![
                (top_level_id(0, 1), 'a'),
                (top_level_id(0, 2), 'b'),
                (top_level_id(0, 3), 'c'),
            ]
        );

        // painting_order() returns only the visible webviews, in painting order.
        let mut painting_order = webviews.painting_order();
        assert_eq!(painting_order.next(), Some((&top_level_id(0, 2), &'b')));
        assert_eq!(painting_order.next(), Some((&top_level_id(0, 1), &'a')));
        assert_eq!(painting_order.next(), None);
        drop(painting_order);

        // remove() removes the given webview from both the map and the painting order.
        assert!(webviews.remove(top_level_id(0, 1)).is_ok());
        assert!(webviews.remove(top_level_id(0, 2)).is_ok());
        assert!(webviews.remove(top_level_id(0, 3)).is_ok());
        assert!(webviews_sorted(&webviews).is_empty());
        assert!(webviews.painting_order.is_empty());
    }
}
