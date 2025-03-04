/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::{Entry, Keys, Values, ValuesMut};
use std::rc::Rc;

use base::id::{PipelineId, WebViewId};
use compositing_traits::SendableFrameTree;
use fnv::FnvHashSet;
use log::debug;
use script_traits::AnimationState;
use webrender_api::units::{DeviceRect, LayoutVector2D};

use crate::IOCompositor;
use crate::compositor::PipelineDetails;

pub(crate) struct WebView {
    /// The [`WebViewId`] of the `WebView` associated with this [`WebViewDetails`].
    pub id: WebViewId,
    /// The root [`PipelineId`] of the currently displayed page in this WebView.
    pub root_pipeline_id: Option<PipelineId>,
    pub rect: DeviceRect,
    /// Tracks details about each active pipeline that the compositor knows about.
    pub pipelines: HashMap<PipelineId, PipelineDetails>,
    /// This is a temporary map between [`PipelineId`]s and their associated [`WebViewId`]. Once
    /// all renderer operations become per-`WebView` this map can be removed, but we still sometimes
    /// need to work backwards to figure out what `WebView` is associated with a `Pipeline`.
    pub pipeline_to_webview_map: Rc<RefCell<HashMap<PipelineId, WebViewId>>>,
}

impl Drop for WebView {
    fn drop(&mut self) {
        self.pipeline_to_webview_map
            .borrow_mut()
            .retain(|_, webview_id| self.id != *webview_id);
    }
}

impl WebView {
    pub(crate) fn animations_or_animation_callbacks_running(&self) -> bool {
        self.pipelines
            .values()
            .any(PipelineDetails::animations_or_animation_callbacks_running)
    }

    pub(crate) fn animation_callbacks_running(&self) -> bool {
        self.pipelines
            .values()
            .any(PipelineDetails::animation_callbacks_running)
    }

    pub(crate) fn pipeline_ids(&self) -> Keys<'_, PipelineId, PipelineDetails> {
        self.pipelines.keys()
    }

    pub(crate) fn pipeline_details(&mut self, pipeline_id: PipelineId) -> &mut PipelineDetails {
        self.pipelines.entry(pipeline_id).or_insert_with(|| {
            self.pipeline_to_webview_map
                .borrow_mut()
                .insert(pipeline_id, self.id);
            PipelineDetails::new(pipeline_id)
        })
    }

    pub(crate) fn set_throttled(&mut self, pipeline_id: PipelineId, throttled: bool) {
        self.pipeline_details(pipeline_id).throttled = throttled;
    }

    pub(crate) fn remove_pipeline(&mut self, pipeline_id: PipelineId) {
        self.pipeline_to_webview_map
            .borrow_mut()
            .remove(&pipeline_id);
        self.pipelines.remove(&pipeline_id);
    }

    pub(crate) fn set_frame_tree(&mut self, frame_tree: &SendableFrameTree) {
        let pipeline_id = frame_tree.pipeline.id;
        let old_pipeline_id = std::mem::replace(&mut self.root_pipeline_id, Some(pipeline_id));

        if old_pipeline_id != self.root_pipeline_id {
            debug!(
                "Updating webview ({:?}) from pipeline {:?} to {:?}",
                3, old_pipeline_id, self.root_pipeline_id
            );
        }

        self.set_frame_tree_on_pipeline_details(frame_tree, None);
        self.reset_scroll_tree_for_unattached_pipelines(frame_tree);
    }

    pub(crate) fn set_frame_tree_on_pipeline_details(
        &mut self,
        frame_tree: &SendableFrameTree,
        parent_pipeline_id: Option<PipelineId>,
    ) {
        let pipeline_id = frame_tree.pipeline.id;
        let pipeline_details = self.pipeline_details(pipeline_id);
        pipeline_details.pipeline = Some(frame_tree.pipeline.clone());
        pipeline_details.parent_pipeline_id = parent_pipeline_id;

        for kid in &frame_tree.children {
            self.set_frame_tree_on_pipeline_details(kid, Some(pipeline_id));
        }
    }

    pub(crate) fn reset_scroll_tree_for_unattached_pipelines(
        &mut self,
        frame_tree: &SendableFrameTree,
    ) {
        // TODO(mrobinson): Eventually this can selectively preserve the scroll trees
        // state for some unattached pipelines in order to preserve scroll position when
        // navigating backward and forward.
        fn collect_pipelines(
            pipelines: &mut FnvHashSet<PipelineId>,
            frame_tree: &SendableFrameTree,
        ) {
            pipelines.insert(frame_tree.pipeline.id);
            for kid in &frame_tree.children {
                collect_pipelines(pipelines, kid);
            }
        }

        let mut attached_pipelines: FnvHashSet<PipelineId> = FnvHashSet::default();
        collect_pipelines(&mut attached_pipelines, frame_tree);

        self.pipelines
            .iter_mut()
            .filter(|(id, _)| !attached_pipelines.contains(id))
            .for_each(|(_, details)| {
                details.scroll_tree.nodes.iter_mut().for_each(|node| {
                    node.set_offset(LayoutVector2D::zero());
                })
            })
    }

    /// Sets or unsets the animations-running flag for the given pipeline, and schedules a
    /// recomposite if necessary. Returns true if the pipeline is throttled.
    pub(crate) fn change_running_animations_state(
        &mut self,
        pipeline_id: PipelineId,
        animation_state: AnimationState,
    ) -> bool {
        let pipeline_details = self.pipeline_details(pipeline_id);
        match animation_state {
            AnimationState::AnimationsPresent => {
                pipeline_details.animations_running = true;
            },
            AnimationState::AnimationCallbacksPresent => {
                pipeline_details.animation_callbacks_running = true;
            },
            AnimationState::NoAnimationsPresent => {
                pipeline_details.animations_running = false;
            },
            AnimationState::NoAnimationCallbacksPresent => {
                pipeline_details.animation_callbacks_running = false;
            },
        }
        pipeline_details.throttled
    }

    pub(crate) fn tick_all_animations(&self, compositor: &IOCompositor) -> bool {
        let mut ticked_any = false;
        for pipeline_details in self.pipelines.values() {
            ticked_any = pipeline_details.tick_animations(compositor) || ticked_any;
        }
        ticked_any
    }

    pub(crate) fn tick_animations_for_pipeline(
        &self,
        pipeline_id: PipelineId,
        compositor: &IOCompositor,
    ) {
        if let Some(pipeline_details) = self.pipelines.get(&pipeline_id) {
            pipeline_details.tick_animations(compositor);
        }
    }

    pub(crate) fn add_pending_paint_metric(&mut self, pipeline_id: PipelineId, epoch: base::Epoch) {
        self.pipeline_details(pipeline_id)
            .pending_paint_metrics
            .push(epoch);
    }
}
#[derive(Debug)]
pub struct WebViewManager<WebView> {
    /// Our top-level browsing contexts. In the WebRender scene, their pipelines are the children of
    /// a single root pipeline that also applies any pinch zoom transformation.
    webviews: HashMap<WebViewId, WebView>,

    /// The order to paint them in, topmost last.
    pub(crate) painting_order: Vec<WebViewId>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnknownWebView(pub WebViewId);

impl<WebView> Default for WebViewManager<WebView> {
    fn default() -> Self {
        Self {
            webviews: Default::default(),
            painting_order: Default::default(),
        }
    }
}

impl<WebView> WebViewManager<WebView> {
    pub fn remove(&mut self, webview_id: WebViewId) -> Result<WebView, UnknownWebView> {
        self.painting_order.retain(|b| *b != webview_id);
        self.webviews
            .remove(&webview_id)
            .ok_or(UnknownWebView(webview_id))
    }

    pub fn get(&self, webview_id: WebViewId) -> Option<&WebView> {
        self.webviews.get(&webview_id)
    }

    pub fn get_mut(&mut self, webview_id: WebViewId) -> Option<&mut WebView> {
        self.webviews.get_mut(&webview_id)
    }

    /// Returns true iff the painting order actually changed.
    pub fn show(&mut self, webview_id: WebViewId) -> Result<bool, UnknownWebView> {
        if !self.webviews.contains_key(&webview_id) {
            return Err(UnknownWebView(webview_id));
        }
        if !self.painting_order.contains(&webview_id) {
            self.painting_order.push(webview_id);
            return Ok(true);
        }
        Ok(false)
    }

    /// Returns true iff the painting order actually changed.
    pub fn hide(&mut self, webview_id: WebViewId) -> Result<bool, UnknownWebView> {
        if !self.webviews.contains_key(&webview_id) {
            return Err(UnknownWebView(webview_id));
        }
        if self.painting_order.contains(&webview_id) {
            self.painting_order.retain(|b| *b != webview_id);
            return Ok(true);
        }
        Ok(false)
    }

    /// Returns true iff the painting order actually changed.
    pub fn hide_all(&mut self) -> bool {
        if !self.painting_order.is_empty() {
            self.painting_order.clear();
            return true;
        }
        false
    }

    /// Returns true iff the painting order actually changed.
    pub fn raise_to_top(&mut self, webview_id: WebViewId) -> Result<bool, UnknownWebView> {
        if !self.webviews.contains_key(&webview_id) {
            return Err(UnknownWebView(webview_id));
        }
        if self.painting_order.last() != Some(&webview_id) {
            self.hide(webview_id)?;
            self.show(webview_id)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn painting_order(&self) -> impl Iterator<Item = (&WebViewId, &WebView)> {
        self.painting_order
            .iter()
            .flat_map(move |webview_id| self.get(*webview_id).map(|b| (webview_id, b)))
    }

    pub fn entry(&mut self, webview_id: WebViewId) -> Entry<'_, WebViewId, WebView> {
        self.webviews.entry(webview_id)
    }

    pub fn iter(&self) -> Values<'_, WebViewId, WebView> {
        self.webviews.values()
    }

    pub fn iter_mut(&mut self) -> ValuesMut<'_, WebViewId, WebView> {
        self.webviews.values_mut()
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU32;

    use base::id::{
        BrowsingContextId, BrowsingContextIndex, PipelineNamespace, PipelineNamespaceId,
        TopLevelBrowsingContextId,
    };

    use crate::webview::{UnknownWebView, WebViewAlreadyExists, WebViewManager};

    fn top_level_id(namespace_id: u32, index: u32) -> TopLevelBrowsingContextId {
        TopLevelBrowsingContextId(BrowsingContextId {
            namespace_id: PipelineNamespaceId(namespace_id),
            index: BrowsingContextIndex(NonZeroU32::new(index).unwrap()),
        })
    }

    fn webviews_sorted<WebView: Clone>(
        webviews: &WebViewManager<WebView>,
    ) -> Vec<(TopLevelBrowsingContextId, WebView)> {
        let mut keys = webviews.webviews.keys().collect::<Vec<_>>();
        keys.sort();
        keys.iter()
            .map(|&id| (*id, webviews.webviews.get(id).cloned().unwrap()))
            .collect()
    }

    #[test]
    fn test() {
        PipelineNamespace::install(PipelineNamespaceId(0));
        let mut webviews = WebViewManager::default();

        // add() adds the webview to the map, but not the painting order.
        assert!(webviews.add(TopLevelBrowsingContextId::new(), 'a').is_ok());
        assert!(webviews.add(TopLevelBrowsingContextId::new(), 'b').is_ok());
        assert!(webviews.add(TopLevelBrowsingContextId::new(), 'c').is_ok());
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
        assert_eq!(
            webviews.add(top_level_id(0, 3), 'd'),
            Err(WebViewAlreadyExists(top_level_id(0, 3)))
        );

        // Other methods return UnknownWebView or None if the webview id doesn’t exist.
        assert_eq!(
            webviews.remove(top_level_id(1, 1)),
            Err(UnknownWebView(top_level_id(1, 1)))
        );
        assert_eq!(webviews.get(top_level_id(1, 1)), None);
        assert_eq!(webviews.get_mut(top_level_id(1, 1)), None);
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
