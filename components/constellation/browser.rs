/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};

use msg::constellation_msg::TopLevelBrowsingContextId;

#[derive(Debug)]
pub struct BrowserManager<Browser> {
    /// Our top-level browsing contexts. In the WebRender scene, their pipelines are the children of
    /// a single root pipeline that also applies any pinch zoom transformation.
    browsers: HashMap<TopLevelBrowsingContextId, Browser>,

    /// The order in which they were focused, latest last.
    focus_order: Vec<TopLevelBrowsingContextId>,

    /// Whether the latest browser in focus order is currently focused.
    is_focused: bool,

    /// The browsers that would be visible in a containing native window.
    visible_browsers: HashSet<TopLevelBrowsingContextId>,

    /// Whether our native window is visible, or true if there is no such window.
    native_window_is_visible: bool,
}

impl<Browser> Default for BrowserManager<Browser> {
    fn default() -> Self {
        Self {
            browsers: HashMap::default(),
            focus_order: Vec::default(),
            is_focused: false,
            visible_browsers: HashSet::default(),
            native_window_is_visible: true,
        }
    }
}

impl<Browser> BrowserManager<Browser> {
    pub fn add(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        browser: Browser,
    ) {
        self.browsers.insert(top_level_browsing_context_id, browser);
    }

    pub fn remove(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) -> Option<Browser> {
        if self.focus_order.last() == Some(&top_level_browsing_context_id) {
            self.is_focused = false;
        }
        self.focus_order
            .retain(|b| *b != top_level_browsing_context_id);
        self.browsers.remove(&top_level_browsing_context_id)
    }

    pub fn get(
        &self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) -> Option<&Browser> {
        self.browsers.get(&top_level_browsing_context_id)
    }

    pub fn get_mut(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) -> Option<&mut Browser> {
        self.browsers.get_mut(&top_level_browsing_context_id)
    }

    pub fn focused_browser(&self) -> Option<(TopLevelBrowsingContextId, &Browser)> {
        if !self.is_focused {
            return None;
        }

        if let Some(top_level_browsing_context_id) = self.focus_order.last().cloned() {
            debug_assert!(
                self.browsers.contains_key(&top_level_browsing_context_id),
                "BUG: browser in .focus_order not in .browsers!",
            );
            self.get(top_level_browsing_context_id)
                .map(|browser| (top_level_browsing_context_id, browser))
        } else {
            debug_assert!(false, "BUG: .is_focused but no browsers in .focus_order!");
            None
        }
    }

    pub fn focus(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        debug_assert!(self.browsers.contains_key(&top_level_browsing_context_id));
        self.focus_order
            .retain(|b| *b != top_level_browsing_context_id);
        self.focus_order.push(top_level_browsing_context_id);
        self.is_focused = true;
    }

    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }

    // TODO Use this when showing or hiding a browser.
    #[allow(unused)]
    pub fn set_browser_visibility(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        visible: bool,
    ) {
        debug_assert!(self.browsers.contains_key(&top_level_browsing_context_id));
        if visible {
            self.visible_browsers.insert(top_level_browsing_context_id);
        } else {
            self.visible_browsers.remove(&top_level_browsing_context_id);
        }
    }

    pub fn set_native_window_visibility(&mut self, visible: bool) {
        self.native_window_is_visible = visible;
    }

    /// Returns true iff the browser is visible and the native window is visible.
    pub fn is_effectively_visible(
        &self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) -> bool {
        debug_assert!(self.browsers.contains_key(&top_level_browsing_context_id));
        self.native_window_is_visible &&
            self.visible_browsers
                .contains(&top_level_browsing_context_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TopLevelBrowsingContextId, &Browser)> {
        self.browsers.iter()
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU32;

    use msg::constellation_msg::{
        BrowsingContextId, BrowsingContextIndex, PipelineNamespace, PipelineNamespaceId,
        TopLevelBrowsingContextId,
    };

    use crate::browser::BrowserManager;

    fn top_level_id(namespace_id: u32, index: u32) -> TopLevelBrowsingContextId {
        TopLevelBrowsingContextId(BrowsingContextId {
            namespace_id: PipelineNamespaceId(namespace_id),
            index: BrowsingContextIndex(NonZeroU32::new(index).expect("Incorrect test case")),
        })
    }

    fn browsers_sorted<Browser: Clone>(
        browsers: &BrowserManager<Browser>,
    ) -> Vec<(TopLevelBrowsingContextId, Browser)> {
        let mut keys = browsers.browsers.keys().collect::<Vec<_>>();
        keys.sort();
        keys.iter()
            .map(|&id| {
                (
                    *id,
                    browsers
                        .browsers
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
        let mut browsers = BrowserManager::default();

        // add() adds the browser to the map, but does not focus it.
        browsers.add(TopLevelBrowsingContextId::new(), 'a');
        browsers.add(TopLevelBrowsingContextId::new(), 'b');
        browsers.add(TopLevelBrowsingContextId::new(), 'c');
        assert_eq!(
            browsers_sorted(&browsers),
            vec![
                (top_level_id(0, 1), 'a'),
                (top_level_id(0, 2), 'b'),
                (top_level_id(0, 3), 'c'),
            ]
        );
        assert!(browsers.focus_order.is_empty());
        assert_eq!(browsers.is_focused, false);

        // focus() makes the given browser the latest in focus order.
        browsers.focus(top_level_id(0, 2));
        assert_eq!(browsers.focus_order, vec![top_level_id(0, 2)]);
        assert_eq!(browsers.is_focused, true);
        browsers.focus(top_level_id(0, 1));
        assert_eq!(
            browsers.focus_order,
            vec![top_level_id(0, 2), top_level_id(0, 1)]
        );
        assert_eq!(browsers.is_focused, true);
        browsers.focus(top_level_id(0, 3));
        assert_eq!(
            browsers.focus_order,
            vec![top_level_id(0, 2), top_level_id(0, 1), top_level_id(0, 3)]
        );
        assert_eq!(browsers.is_focused, true);

        // unfocus() clears the “is focused” flag, but does not touch the focus order.
        browsers.unfocus();
        assert_eq!(
            browsers.focus_order,
            vec![top_level_id(0, 2), top_level_id(0, 1), top_level_id(0, 3)]
        );
        assert_eq!(browsers.is_focused, false);

        // focus() avoids duplicates in focus order, when the given browser has been focused before.
        browsers.focus(top_level_id(0, 1));
        assert_eq!(
            browsers.focus_order,
            vec![top_level_id(0, 2), top_level_id(0, 3), top_level_id(0, 1)]
        );
        assert_eq!(browsers.is_focused, true);

        // is_effectively_visible() checks that the given browser is visible.
        browsers.set_browser_visibility(top_level_id(0, 1), true);
        browsers.set_browser_visibility(top_level_id(0, 3), true);
        assert_eq!(browsers.is_effectively_visible(top_level_id(0, 1)), true);
        assert_eq!(browsers.is_effectively_visible(top_level_id(0, 2)), false);
        assert_eq!(browsers.is_effectively_visible(top_level_id(0, 3)), true);

        // is_effectively_visible() checks that the native window is visible.
        browsers.set_native_window_visibility(false);
        assert_eq!(browsers.is_effectively_visible(top_level_id(0, 1)), false);
        assert_eq!(browsers.is_effectively_visible(top_level_id(0, 2)), false);
        assert_eq!(browsers.is_effectively_visible(top_level_id(0, 3)), false);

        // set_native_window_visibility() does not destroy or prevent changes to browser visibility state.
        browsers.set_browser_visibility(top_level_id(0, 1), false);
        browsers.set_native_window_visibility(true);
        assert_eq!(browsers.is_effectively_visible(top_level_id(0, 1)), false);
        assert_eq!(browsers.is_effectively_visible(top_level_id(0, 2)), false);
        assert_eq!(browsers.is_effectively_visible(top_level_id(0, 3)), true);

        // remove() clears the “is focused” flag iff the given browser was focused.
        browsers.remove(top_level_id(0, 2));
        assert_eq!(browsers.is_focused, true);
        browsers.remove(top_level_id(0, 1));
        assert_eq!(browsers.is_focused, false);
        browsers.remove(top_level_id(0, 3));
        assert_eq!(browsers.is_focused, false);

        // remove() removes the given browser from both the map and the focus order.
        assert!(browsers_sorted(&browsers).is_empty());
        assert!(browsers.focus_order.is_empty());
    }
}
