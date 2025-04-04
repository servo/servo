/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use base::id::{BrowsingContextId, PipelineId, WebViewId};

#[derive(Debug, Default)]
pub(crate) struct IdMap {
    pub(crate) browser_ids: HashMap<WebViewId, u32>,
    pub(crate) browsing_context_ids: HashMap<BrowsingContextId, u32>,
    pub(crate) outer_window_ids: HashMap<PipelineId, u32>,
}

impl IdMap {
    pub(crate) fn browser_id(&mut self, webview_id: WebViewId) -> DevtoolsBrowserId {
        let len = self
            .browser_ids
            .len()
            .checked_add(1)
            .expect("WebViewId count overflow")
            .try_into()
            .expect("DevtoolsBrowserId overflow");
        DevtoolsBrowserId(*self.browser_ids.entry(webview_id).or_insert(len))
    }
    pub(crate) fn browsing_context_id(
        &mut self,
        browsing_context_id: BrowsingContextId,
    ) -> DevtoolsBrowsingContextId {
        let len = self
            .browsing_context_ids
            .len()
            .checked_add(1)
            .expect("BrowsingContextId count overflow")
            .try_into()
            .expect("DevtoolsBrowsingContextId overflow");
        DevtoolsBrowsingContextId(
            *self
                .browsing_context_ids
                .entry(browsing_context_id)
                .or_insert(len),
        )
    }
    pub(crate) fn outer_window_id(&mut self, pipeline_id: PipelineId) -> DevtoolsOuterWindowId {
        let len = self
            .outer_window_ids
            .len()
            .checked_add(1)
            .expect("PipelineId count overflow")
            .try_into()
            .expect("DevtoolsOuterWindowId overflow");
        DevtoolsOuterWindowId(*self.outer_window_ids.entry(pipeline_id).or_insert(len))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DevtoolsBrowserId(u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DevtoolsBrowsingContextId(u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DevtoolsOuterWindowId(u32);

impl DevtoolsBrowserId {
    pub(crate) fn value(&self) -> u32 {
        self.0
    }
}

impl DevtoolsBrowsingContextId {
    pub(crate) fn value(&self) -> u32 {
        self.0
    }
}

impl DevtoolsOuterWindowId {
    pub(crate) fn value(&self) -> u32 {
        self.0
    }
}

#[test]
pub(crate) fn test_id_map() {
    use std::thread;

    use base::id::{PipelineNamespace, PipelineNamespaceId};
    use crossbeam_channel::unbounded;

    macro_rules! test_sequential_id_assignment {
        ($id_type:ident, $new_id_function:expr, $map_id_function:expr) => {
            let (sender, receiver) = unbounded();
            let sender1 = sender.clone();
            let sender2 = sender.clone();
            let sender3 = sender.clone();
            let threads = [
                thread::spawn(move || {
                    PipelineNamespace::install(PipelineNamespaceId(1));
                    sender1.send($new_id_function()).expect("Send failed");
                    sender1.send($new_id_function()).expect("Send failed");
                    sender1.send($new_id_function()).expect("Send failed");
                }),
                thread::spawn(move || {
                    PipelineNamespace::install(PipelineNamespaceId(2));
                    sender2.send($new_id_function()).expect("Send failed");
                    sender2.send($new_id_function()).expect("Send failed");
                    sender2.send($new_id_function()).expect("Send failed");
                }),
                thread::spawn(move || {
                    PipelineNamespace::install(PipelineNamespaceId(3));
                    sender3.send($new_id_function()).expect("Send failed");
                    sender3.send($new_id_function()).expect("Send failed");
                    sender3.send($new_id_function()).expect("Send failed");
                }),
            ];
            for thread in threads {
                thread.join().expect("Thread join failed");
            }
            let mut id_map = IdMap::default();
            assert_eq!(
                $map_id_function(&mut id_map, receiver.recv().expect("Recv failed")),
                $id_type(1)
            );
            assert_eq!(
                $map_id_function(&mut id_map, receiver.recv().expect("Recv failed")),
                $id_type(2)
            );
            assert_eq!(
                $map_id_function(&mut id_map, receiver.recv().expect("Recv failed")),
                $id_type(3)
            );
            assert_eq!(
                $map_id_function(&mut id_map, receiver.recv().expect("Recv failed")),
                $id_type(4)
            );
            assert_eq!(
                $map_id_function(&mut id_map, receiver.recv().expect("Recv failed")),
                $id_type(5)
            );
            assert_eq!(
                $map_id_function(&mut id_map, receiver.recv().expect("Recv failed")),
                $id_type(6)
            );
            assert_eq!(
                $map_id_function(&mut id_map, receiver.recv().expect("Recv failed")),
                $id_type(7)
            );
            assert_eq!(
                $map_id_function(&mut id_map, receiver.recv().expect("Recv failed")),
                $id_type(8)
            );
            assert_eq!(
                $map_id_function(&mut id_map, receiver.recv().expect("Recv failed")),
                $id_type(9)
            );
        };
    }

    test_sequential_id_assignment!(
        DevtoolsBrowserId,
        || WebViewId::new(),
        |id_map: &mut IdMap, id| id_map.browser_id(id)
    );
    test_sequential_id_assignment!(
        DevtoolsBrowsingContextId,
        || BrowsingContextId::new(),
        |id_map: &mut IdMap, id| id_map.browsing_context_id(id)
    );
    test_sequential_id_assignment!(
        DevtoolsOuterWindowId,
        || PipelineId::new(),
        |id_map: &mut IdMap, id| id_map.outer_window_id(id)
    );
}
