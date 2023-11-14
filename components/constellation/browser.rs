use std::collections::HashMap;

use msg::constellation_msg::{TopLevelBrowsingContextId, PipelineId};

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
        self.focus_order.retain(|b| *b != top_level_browsing_context_id);
        self.is_focused = false;
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
