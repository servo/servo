use std::collections::HashMap;

use msg::constellation_msg::{TopLevelBrowsingContextId, PipelineId, PipelineNamespaceId, PipelineNamespace};

#[derive(Debug)]
pub struct BrowserManager<Browser> {
    /// Our top-level browsing contexts. In the WebRender scene, their pipelines are the children of
    /// a single root pipeline that also applies any pinch zoom transformation.
    browsers: HashMap<TopLevelBrowsingContextId, Browser>,

    /// The order in which they were focused, latest last.
    focus_order: Vec<TopLevelBrowsingContextId>,

    /// Whether the latest browser in focus order is currently focused.
    is_focused: bool,
}

impl<Browser> Default for BrowserManager<Browser> {
    fn default() -> Self {
        Self { browsers: Default::default(), focus_order: Default::default(), is_focused: Default::default() }
    }
}

impl<Browser> BrowserManager<Browser> {
    pub fn add(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId, browser: Browser) {
        self.browsers.insert(top_level_browsing_context_id, browser);
    }

    pub fn remove(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) -> Option<Browser> {
        if self.focus_order.last() == Some(&top_level_browsing_context_id) {
            self.is_focused = false;
        }
        self.focus_order.retain(|b| *b != top_level_browsing_context_id);
        self.browsers.remove(&top_level_browsing_context_id)
    }

    pub fn get(&self, top_level_browsing_context_id: TopLevelBrowsingContextId) -> Option<&Browser> {
        self.browsers.get(&top_level_browsing_context_id)
    }

    pub fn get_mut(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) -> Option<&mut Browser> {
        self.browsers.get_mut(&top_level_browsing_context_id)
    }

    pub fn focus(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        debug_assert!(self.browsers.contains_key(&top_level_browsing_context_id));
        self.focus_order.retain(|b| *b != top_level_browsing_context_id);
        self.focus_order.push(top_level_browsing_context_id);
        self.is_focused = true;
    }

    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }
}


#[cfg(test)] fn top_level_id(namespace_id: u32, index: u32) -> TopLevelBrowsingContextId {
    use std::num::NonZeroU32;

    use msg::constellation_msg::{BrowsingContextIndex, BrowsingContextId};

    TopLevelBrowsingContextId(BrowsingContextId {
        namespace_id: PipelineNamespaceId(namespace_id),
        index: BrowsingContextIndex(NonZeroU32::new(index).unwrap()),
    })
}

#[cfg(test)] fn browsers_sorted<Browser: Clone>(browsers: &BrowserManager<Browser>) -> Vec<(TopLevelBrowsingContextId, Browser)> {
    let mut keys = browsers.browsers.keys().collect::<Vec<_>>();
    keys.sort();
    keys.iter()
        .map(|&id| (*id, browsers.browsers.get(id).cloned().unwrap()))
        .collect()
}

#[test] fn test() {
    PipelineNamespace::install(PipelineNamespaceId(0));
    let mut browsers = BrowserManager::default();

    // add() adds the browser to the map, but does not focus it.
    browsers.add(TopLevelBrowsingContextId::new(), 'a');
    browsers.add(TopLevelBrowsingContextId::new(), 'b');
    browsers.add(TopLevelBrowsingContextId::new(), 'c');
    assert_eq!(browsers_sorted(&browsers), vec![
        (top_level_id(0, 1), 'a'),
        (top_level_id(0, 2), 'b'),
        (top_level_id(0, 3), 'c'),
    ]);
    assert!(browsers.focus_order.is_empty());
    assert_eq!(browsers.is_focused, false);

    // focus() makes the given browser the latest in focus order.
    browsers.focus(top_level_id(0, 2));
    assert_eq!(browsers.focus_order, vec![top_level_id(0, 2)]);
    assert_eq!(browsers.is_focused, true);
    browsers.focus(top_level_id(0, 1));
    assert_eq!(browsers.focus_order, vec![top_level_id(0, 2), top_level_id(0, 1)]);
    assert_eq!(browsers.is_focused, true);
    browsers.focus(top_level_id(0, 3));
    assert_eq!(browsers.focus_order, vec![top_level_id(0, 2), top_level_id(0, 1), top_level_id(0, 3)]);
    assert_eq!(browsers.is_focused, true);

    // unfocus() clears the “is focused” flag, but does not touch the focus order.
    browsers.unfocus();
    assert_eq!(browsers.focus_order, vec![top_level_id(0, 2), top_level_id(0, 1), top_level_id(0, 3)]);
    assert_eq!(browsers.is_focused, false);

    // focus() avoids duplicates in focus order, when the given browser has been focused before.
    browsers.focus(top_level_id(0, 1));
    assert_eq!(browsers.focus_order, vec![top_level_id(0, 2), top_level_id(0, 3), top_level_id(0, 1)]);
    assert_eq!(browsers.is_focused, true);

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
