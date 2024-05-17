/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use base::id::{PipelineId, WebViewId};
use webrender_api::units::DeviceRect;

#[derive(Debug, Default)]
pub struct WebView {
    pub pipeline_id: Option<PipelineId>,
    pub rect: DeviceRect,
}

#[derive(Debug, Default)]
pub struct WebViewManager<WebView> {
    /// Our top-level browsing contexts. In the WebRender scene, their pipelines are the children of
    /// a single root pipeline that also applies any pinch zoom transformation.
    webviews: HashMap<WebViewId, WebView>,

    /// The order to paint them in, topmost last.
    painting_order: Vec<WebViewId>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WebViewAlreadyExists(pub WebViewId);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnknownWebView(pub WebViewId);

impl<WebView> WebViewManager<WebView> {
    pub fn add(
        &mut self,
        webview_id: WebViewId,
        webview: WebView,
    ) -> Result<&mut WebView, WebViewAlreadyExists> {
        if self.webviews.contains_key(&webview_id) {
            return Err(WebViewAlreadyExists(webview_id));
        }
        Ok(self.webviews.entry(webview_id).or_insert(webview))
    }

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
