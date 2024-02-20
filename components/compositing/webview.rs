/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use msg::constellation_msg::TopLevelBrowsingContextId;

#[derive(Debug)]
pub struct WebViewManager<WebView> {
    /// Our top-level browsing contexts. In the WebRender scene, their pipelines are the children of
    /// a single root pipeline that also applies any pinch zoom transformation.
    webviews: HashMap<TopLevelBrowsingContextId, WebView>,

    /// The order to paint them in, topmost last.
    painting_order: Vec<TopLevelBrowsingContextId>,
}

impl<WebView> Default for WebViewManager<WebView> {
    fn default() -> Self {
        Self {
            webviews: Default::default(),
            painting_order: Default::default(),
        }
    }
}

impl<WebView> WebViewManager<WebView> {
    pub fn add(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        webview: WebView
    ) {
        debug_assert!(!self.webviews.contains_key(&top_level_browsing_context_id));
        self.webviews.insert(top_level_browsing_context_id, webview);
    }

    pub fn remove(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) -> Option<WebView> {
        self.painting_order
            .retain(|b| *b != top_level_browsing_context_id);
        self.webviews.remove(&top_level_browsing_context_id)
    }

    pub fn get(
        &self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) -> Option<&WebView> {
        self.webviews.get(&top_level_browsing_context_id)
    }

    pub fn get_mut(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) -> Option<&mut WebView> {
        self.webviews.get_mut(&top_level_browsing_context_id)
    }

    pub fn show(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        debug_assert!(self.webviews.contains_key(&top_level_browsing_context_id));
        if !self.painting_order.contains(&top_level_browsing_context_id) {
            self.painting_order.push(top_level_browsing_context_id);
        }
    }

    pub fn hide(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        debug_assert!(self.webviews.contains_key(&top_level_browsing_context_id));
        self.painting_order
            .retain(|b| *b != top_level_browsing_context_id);
    }

    pub fn raise_to_top(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        self.hide(top_level_browsing_context_id);
        self.show(top_level_browsing_context_id);
    }

    pub fn painting_order(&self) -> impl Iterator<Item = (&TopLevelBrowsingContextId, &WebView)> {
        self.painting_order
            .iter()
            .flat_map(move |browser_id| self.webviews.get(browser_id).map(|b| (browser_id, b)))
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU32;

    use msg::constellation_msg::{
        BrowsingContextId, BrowsingContextIndex, PipelineNamespace, PipelineNamespaceId,
        TopLevelBrowsingContextId,
    };

    use crate::webview::WebViewManager;

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
        let mut browsers = WebViewManager::default();

        // add() adds the browser to the map, but not the painting order.
        browsers.add(TopLevelBrowsingContextId::new(), 'a');
        browsers.add(TopLevelBrowsingContextId::new(), 'b');
        browsers.add(TopLevelBrowsingContextId::new(), 'c');
        assert_eq!(
            webviews_sorted(&browsers),
            vec![
                (top_level_id(0, 1), 'a'),
                (top_level_id(0, 2), 'b'),
                (top_level_id(0, 3), 'c'),
            ]
        );
        assert!(browsers.painting_order.is_empty());

        // For browsers not yet visible, both show() and raise_to_top() add the given browser on top.
        browsers.show(top_level_id(0, 2));
        assert_eq!(browsers.painting_order, vec![top_level_id(0, 2)]);
        browsers.raise_to_top(top_level_id(0, 1));
        assert_eq!(
            browsers.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1)]
        );
        browsers.show(top_level_id(0, 3));
        assert_eq!(
            browsers.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1), top_level_id(0, 3)]
        );

        // For browsers already visible, show() does nothing, while raise_to_top() makes it on top.
        browsers.show(top_level_id(0, 1));
        assert_eq!(
            browsers.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1), top_level_id(0, 3)]
        );
        browsers.raise_to_top(top_level_id(0, 1));
        assert_eq!(
            browsers.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 3), top_level_id(0, 1)]
        );

        // hide() removes the browser from the painting order, but not the map.
        browsers.hide(top_level_id(0, 3));
        assert_eq!(
            browsers.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1)]
        );
        assert_eq!(
            webviews_sorted(&browsers),
            vec![
                (top_level_id(0, 1), 'a'),
                (top_level_id(0, 2), 'b'),
                (top_level_id(0, 3), 'c'),
            ]
        );

        // painting_order() returns only the visible browsers, in painting order.
        let mut painting_order = browsers.painting_order();
        assert_eq!(painting_order.next(), Some((&top_level_id(0, 2), &'b')));
        assert_eq!(painting_order.next(), Some((&top_level_id(0, 1), &'a')));
        assert_eq!(painting_order.next(), None);
        drop(painting_order);

        // remove() removes the given browser from both the map and the painting order.
        browsers.remove(top_level_id(0, 1));
        browsers.remove(top_level_id(0, 2));
        browsers.remove(top_level_id(0, 3));
        assert!(webviews_sorted(&browsers).is_empty());
        assert!(browsers.painting_order.is_empty());
    }
}

