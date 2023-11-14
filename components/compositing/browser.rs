use std::collections::HashMap;

use msg::constellation_msg::{TopLevelBrowsingContextId, PipelineId};

#[derive(Debug, Default)]
pub struct BrowserManager {
    /// Our top-level browsing contexts. In the WebRender scene, their pipelines are the children of
    /// a single root pipeline that also applies any pinch zoom transformation.
    browsers: HashMap<TopLevelBrowsingContextId, Browser>,

    /// The order to paint them in, topmost last.
    painting_order: Vec<TopLevelBrowsingContextId>,
}

#[derive(Debug)]
pub struct Browser {
    pub pipeline_id: Option<PipelineId>,
}

impl BrowserManager {
    pub fn add(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId, pipeline_id: Option<PipelineId>) {
        self.browsers.insert(top_level_browsing_context_id, Browser { pipeline_id });
    }

    pub fn remove(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) -> Option<Browser> {
        self.painting_order.retain(|b| *b != top_level_browsing_context_id);
        self.browsers.remove(&top_level_browsing_context_id)
    }

    pub fn get(&self, top_level_browsing_context_id: TopLevelBrowsingContextId) -> Option<&Browser> {
        self.browsers.get(&top_level_browsing_context_id)
    }

    pub fn get_mut(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) -> Option<&mut Browser> {
        self.browsers.get_mut(&top_level_browsing_context_id)
    }

    pub fn show(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        debug_assert!(self.browsers.contains_key(&top_level_browsing_context_id));
        if !self.painting_order.contains(&top_level_browsing_context_id) {
            self.painting_order.push(top_level_browsing_context_id);
        }
    }

    pub fn hide(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        debug_assert!(self.browsers.contains_key(&top_level_browsing_context_id));
        self.painting_order.retain(|b| *b != top_level_browsing_context_id);
    }

    pub fn raise_to_top(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        self.hide(top_level_browsing_context_id);
        self.show(top_level_browsing_context_id);
    }

    pub fn painting_order(&self) -> impl Iterator<Item = (&TopLevelBrowsingContextId, &Browser)> {
        self.painting_order.iter()
            .flat_map(move |browser_id| self.browsers.get(browser_id).map(|b| (browser_id, b)))
    }
}
