/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, hash_map};

use base::id::{BrowsingContextId, PipelineId};

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::HashMapTracedValues;
use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::window::Window;

/// The collection of all [`Document`]s managed by the [`crate::script_thread::ScriptThread`].
/// This is stored as a mapping of [`PipelineId`] to [`Document`], but for updating the
/// rendering, [`Document`]s should be processed in order via [`Self::documents_in_order`].
#[derive(JSTraceable)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentCollection {
    map: HashMapTracedValues<PipelineId, Dom<Document>>,
}

impl DocumentCollection {
    pub(crate) fn insert(&mut self, pipeline_id: PipelineId, doc: &Document) {
        self.map.insert(pipeline_id, Dom::from_ref(doc));
    }

    pub(crate) fn remove(&mut self, pipeline_id: PipelineId) -> Option<DomRoot<Document>> {
        self.map
            .remove(&pipeline_id)
            .map(|ref doc| DomRoot::from_ref(&**doc))
    }

    pub(crate) fn find_document(&self, pipeline_id: PipelineId) -> Option<DomRoot<Document>> {
        self.map
            .get(&pipeline_id)
            .map(|doc| DomRoot::from_ref(&**doc))
    }

    pub(crate) fn find_window(&self, pipeline_id: PipelineId) -> Option<DomRoot<Window>> {
        self.find_document(pipeline_id)
            .map(|doc| DomRoot::from_ref(doc.window()))
    }

    pub(crate) fn find_global(&self, pipeline_id: PipelineId) -> Option<DomRoot<GlobalScope>> {
        self.find_window(pipeline_id)
            .map(|window| DomRoot::from_ref(window.upcast()))
    }

    pub(crate) fn find_iframe(
        &self,
        pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
    ) -> Option<DomRoot<HTMLIFrameElement>> {
        self.find_document(pipeline_id).and_then(|document| {
            document
                .iframes()
                .get(browsing_context_id)
                .map(|iframe| iframe.element.as_rooted())
        })
    }

    pub(crate) fn iter(&self) -> DocumentsIter<'_> {
        DocumentsIter {
            iter: self.map.iter(),
        }
    }

    /// Return the documents managed by this [`crate::script_thread::ScriptThread`] in the
    /// order specified by the *[update the rendering][update-the-rendering]* step of the
    /// HTML specification:
    ///
    /// > Let docs be all fully active Document objects whose relevant agent's event loop is
    /// > eventLoop, sorted arbitrarily except that the following conditions must be met:
    /// >
    /// > Any Document B whose container document is A must be listed after A in the list.
    /// >
    /// > If there are two documents A and B that both have the same non-null container
    /// > document C, then the order of A and B in the list must match the shadow-including
    /// > tree order of their respective navigable containers in C's node tree.
    /// >
    /// > In the steps below that iterate over docs, each Document must be processed in the
    /// > order it is found in the list.
    ///
    /// [update-the-rendering]: https://html.spec.whatwg.org/multipage/#update-the-rendering
    pub(crate) fn documents_in_order(&self) -> Vec<PipelineId> {
        DocumentTree::new(self).documents_in_order()
    }
}

impl Default for DocumentCollection {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn default() -> Self {
        Self {
            map: HashMapTracedValues::new(),
        }
    }
}

#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(crate) struct DocumentsIter<'a> {
    iter: hash_map::Iter<'a, PipelineId, Dom<Document>>,
}

impl Iterator for DocumentsIter<'_> {
    type Item = (PipelineId, DomRoot<Document>);

    fn next(&mut self) -> Option<(PipelineId, DomRoot<Document>)> {
        self.iter
            .next()
            .map(|(id, doc)| (*id, DomRoot::from_ref(&**doc)))
    }
}

#[derive(Default)]
struct DocumentTreeNode {
    parent: Option<PipelineId>,
    children: Vec<PipelineId>,
}

/// A tree representation of [`Document`]s managed by the [`ScriptThread`][st], which is used
/// to generate an ordered set of [`Document`]s for the *update the rendering* step of the
/// HTML5 specification.
///
/// FIXME: The [`ScriptThread`][st] only has a view of [`Document`]s managed by itself,
/// so if there are interceding iframes managed by other `ScriptThread`s, then the
/// order of the [`Document`]s may not be correct. Perhaps the Constellation could
/// ensure that every [`ScriptThread`][st] has the full view of the frame tree.
///
/// [st]: crate::script_thread::ScriptThread
#[derive(Default)]
struct DocumentTree {
    tree: HashMap<PipelineId, DocumentTreeNode>,
}

impl DocumentTree {
    fn new(documents: &DocumentCollection) -> Self {
        let mut tree = DocumentTree::default();
        for (id, document) in documents.iter() {
            let children: Vec<PipelineId> = document
                .iframes()
                .iter()
                .filter_map(|iframe| iframe.pipeline_id())
                .filter(|iframe_pipeline_id| documents.find_document(*iframe_pipeline_id).is_some())
                .collect();
            for child in &children {
                tree.tree.entry(*child).or_default().parent = Some(id);
            }
            tree.tree.entry(id).or_default().children = children;
        }
        tree
    }

    fn documents_in_order(&self) -> Vec<PipelineId> {
        let mut list = Vec::new();
        for (id, node) in self.tree.iter() {
            if node.parent.is_none() {
                self.process_node_for_documents_in_order(*id, &mut list);
            }
        }
        list
    }

    fn process_node_for_documents_in_order(&self, id: PipelineId, list: &mut Vec<PipelineId>) {
        list.push(id);
        for child in self
            .tree
            .get(&id)
            .expect("Should have found child node")
            .children
            .iter()
        {
            self.process_node_for_documents_in_order(*child, list);
        }
    }
}
