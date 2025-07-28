/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use base::id::WebViewId;

#[derive(Debug)]
pub struct WebViewManager<WebView> {
    /// Our top-level browsing contexts. In the WebRender scene, their pipelines are the children of
    /// a single root pipeline that also applies any pinch zoom transformation.
    webviews: HashMap<WebViewId, WebView>,

    /// The order in which they were focused, latest last.
    focus_order: Vec<WebViewId>,

    /// Whether the latest webview in focus order is currently focused.
    is_focused: bool,
}

impl<WebView> Default for WebViewManager<WebView> {
    fn default() -> Self {
        Self {
            webviews: HashMap::default(),
            focus_order: Vec::default(),
            is_focused: false,
        }
    }
}

impl<WebView> WebViewManager<WebView> {
    pub fn add(&mut self, webview_id: WebViewId, webview: WebView) {
        self.webviews.insert(webview_id, webview);
    }

    pub fn remove(&mut self, webview_id: WebViewId) -> Option<WebView> {
        if self.focus_order.last() == Some(&webview_id) {
            self.is_focused = false;
        }
        self.focus_order.retain(|b| *b != webview_id);
        self.webviews.remove(&webview_id)
    }

    pub fn get(&self, webview_id: WebViewId) -> Option<&WebView> {
        self.webviews.get(&webview_id)
    }

    pub fn get_mut(&mut self, webview_id: WebViewId) -> Option<&mut WebView> {
        self.webviews.get_mut(&webview_id)
    }

    pub fn focused_webview(&self) -> Option<(WebViewId, &WebView)> {
        if !self.is_focused {
            return None;
        }

        if let Some(webview_id) = self.focus_order.last().cloned() {
            debug_assert!(
                self.webviews.contains_key(&webview_id),
                "BUG: webview in .focus_order not in .webviews!",
            );
            self.get(webview_id).map(|webview| (webview_id, webview))
        } else {
            debug_assert!(false, "BUG: .is_focused but no webviews in .focus_order!");
            None
        }
    }

    pub fn focus(&mut self, webview_id: WebViewId) -> Result<(), ()> {
        if !self.webviews.contains_key(&webview_id) {
            return Err(());
        }
        self.focus_order.retain(|b| *b != webview_id);
        self.focus_order.push(webview_id);
        self.is_focused = true;
        Ok(())
    }

    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }
}

#[cfg(test)]
mod test {
    use base::id::{BrowsingContextId, Index, PipelineNamespace, PipelineNamespaceId, WebViewId};

    use crate::webview_manager::WebViewManager;

    fn id(namespace_id: u32, index: u32) -> WebViewId {
        WebViewId(BrowsingContextId {
            namespace_id: PipelineNamespaceId(namespace_id),
            index: Index::new(index).expect("Incorrect test case"),
        })
    }

    fn webviews_sorted<WebView: Clone>(
        webviews: &WebViewManager<WebView>,
    ) -> Vec<(WebViewId, WebView)> {
        let mut keys = webviews.webviews.keys().collect::<Vec<_>>();
        keys.sort();
        keys.iter()
            .map(|&id| {
                (
                    *id,
                    webviews
                        .webviews
                        .get(id)
                        .cloned()
                        .expect("Incorrect test case"),
                )
            })
            .collect()
    }

    #[test]
    fn test() {
        PipelineNamespace::install(PipelineNamespaceId(0));
        let mut webviews = WebViewManager::default();

        // add() adds the webview to the map, but does not focus it.
        webviews.add(WebViewId::new(), 'a');
        webviews.add(WebViewId::new(), 'b');
        webviews.add(WebViewId::new(), 'c');
        assert_eq!(
            webviews_sorted(&webviews),
            vec![(id(0, 1), 'a'), (id(0, 2), 'b'), (id(0, 3), 'c'),]
        );
        assert!(webviews.focus_order.is_empty());
        assert_eq!(webviews.is_focused, false);

        // focus() makes the given webview the latest in focus order.
        webviews.focus(id(0, 2));
        assert_eq!(webviews.focus_order, vec![id(0, 2)]);
        assert_eq!(webviews.is_focused, true);
        webviews.focus(id(0, 1));
        assert_eq!(webviews.focus_order, vec![id(0, 2), id(0, 1)]);
        assert_eq!(webviews.is_focused, true);
        webviews.focus(id(0, 3));
        assert_eq!(webviews.focus_order, vec![id(0, 2), id(0, 1), id(0, 3)]);
        assert_eq!(webviews.is_focused, true);

        // unfocus() clears the “is focused” flag, but does not touch the focus order.
        webviews.unfocus();
        assert_eq!(webviews.focus_order, vec![id(0, 2), id(0, 1), id(0, 3)]);
        assert_eq!(webviews.is_focused, false);

        // focus() avoids duplicates in focus order, when the given webview has been focused before.
        webviews.focus(id(0, 1));
        assert_eq!(webviews.focus_order, vec![id(0, 2), id(0, 3), id(0, 1)]);
        assert_eq!(webviews.is_focused, true);

        // remove() clears the “is focused” flag iff the given webview was focused.
        webviews.remove(id(1, 1));
        assert_eq!(webviews.is_focused, true);
        webviews.remove(id(1, 2));
        assert_eq!(webviews.is_focused, true);
        webviews.remove(id(2, 1));
        assert_eq!(webviews.is_focused, true);
        webviews.remove(id(2, 2));
        assert_eq!(webviews.is_focused, true);
        webviews.remove(id(2, 3));
        assert_eq!(webviews.is_focused, true);
        webviews.remove(id(2, 4));
        assert_eq!(webviews.is_focused, true);
        webviews.remove(id(3, 1));
        assert_eq!(webviews.is_focused, true);
        webviews.remove(id(4, 1));
        assert_eq!(webviews.is_focused, true);
        webviews.remove(id(0, 2));
        assert_eq!(webviews.is_focused, true);
        webviews.remove(id(0, 1));
        assert_eq!(webviews.is_focused, false);
        webviews.remove(id(0, 3));
        assert_eq!(webviews.is_focused, false);

        // remove() removes the given webview from both the map and the focus order.
        assert!(webviews_sorted(&webviews).is_empty());
        assert!(webviews.focus_order.is_empty());
    }
}
