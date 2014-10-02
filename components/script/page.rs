/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable};
use dom::bindings::trace::{Traceable, Untraceable};
use dom::bindings::utils::GlobalStaticData;
use dom::document::{Document, DocumentHelpers};
use dom::element::Element;
use dom::node::{Node, NodeHelpers};
use dom::window::Window;
use layout_interface::{DocumentDamage, ReflowForDisplay};
use layout_interface::{DocumentDamageLevel, HitTestResponse, MouseOverResponse};
use layout_interface::{GetRPCMsg, LayoutChan, LayoutRPC};
use layout_interface::{Reflow, ReflowGoal, ReflowMsg};
use layout_interface::UntrustedNodeAddress;
use script_traits::ScriptControlChan;

use geom::point::Point2D;
use js::rust::Cx;
use servo_msg::compositor_msg::PerformingLayout;
use servo_msg::compositor_msg::ScriptListener;
use servo_msg::constellation_msg::{ConstellationChan, WindowSizeData};
use servo_msg::constellation_msg::{PipelineId, SubpageId};
use servo_net::resource_task::ResourceTask;
use servo_util::str::DOMString;
use std::cell::{Cell, RefCell, Ref, RefMut};
use std::comm::{channel, Receiver, Empty, Disconnected};
use std::mem::replace;
use std::rc::Rc;
use url::Url;

/// Encapsulates a handle to a frame and its associated layout information.
#[jstraceable]
pub struct Page {
    /// Pipeline id associated with this page.
    pub id: PipelineId,

    /// Subpage id associated with this page, if any.
    pub subpage_id: Option<SubpageId>,

    /// Unique id for last reflow request; used for confirming completion reply.
    pub last_reflow_id: Traceable<Cell<uint>>,

    /// The outermost frame containing the document, window, and page URL.
    pub frame: Traceable<RefCell<Option<Frame>>>,

    /// A handle for communicating messages to the layout task.
    pub layout_chan: Untraceable<LayoutChan>,

    /// A handle to perform RPC calls into the layout, quickly.
    layout_rpc: Untraceable<Box<LayoutRPC+'static>>,

    /// The port that we will use to join layout. If this is `None`, then layout is not running.
    pub layout_join_port: Untraceable<RefCell<Option<Receiver<()>>>>,

    /// What parts of the document are dirty, if any.
    damage: Traceable<RefCell<Option<DocumentDamage>>>,

    /// The current size of the window, in pixels.
    pub window_size: Traceable<Cell<WindowSizeData>>,

    js_info: Traceable<RefCell<Option<JSPageInfo>>>,

    /// Cached copy of the most recent url loaded by the script
    /// TODO(tkuehn): this currently does not follow any particular caching policy
    /// and simply caches pages forever (!). The bool indicates if reflow is required
    /// when reloading.
    url: Untraceable<RefCell<Option<(Url, bool)>>>,

    next_subpage_id: Traceable<Cell<SubpageId>>,

    /// Pending resize event, if any.
    pub resize_event: Untraceable<Cell<Option<WindowSizeData>>>,

    /// Pending scroll to fragment event, if any
    pub fragment_name: RefCell<Option<String>>,

    /// Associated resource task for use by DOM objects like XMLHttpRequest
    pub resource_task: Untraceable<ResourceTask>,

    /// A handle for communicating messages to the constellation task.
    pub constellation_chan: Untraceable<ConstellationChan>,

    // Child Pages.
    pub children: Traceable<RefCell<Vec<Rc<Page>>>>,

    /// Number of pending reflows that were sent while layout was active.
    pub pending_reflows: Cell<int>,

    /// Number of unnecessary potential reflows that were skipped since the last reflow
    pub avoided_reflows: Cell<int>,
}

pub struct PageIterator {
    stack: Vec<Rc<Page>>,
}

pub trait IterablePage {
    fn iter(&self) -> PageIterator;
    fn find(&self, id: PipelineId) -> Option<Rc<Page>>;
}

impl IterablePage for Rc<Page> {
    fn iter(&self) -> PageIterator {
        PageIterator {
            stack: vec!(self.clone()),
        }
    }
    fn find(&self, id: PipelineId) -> Option<Rc<Page>> {
        if self.id == id { return Some(self.clone()); }
        for page in self.children.deref().borrow().iter() {
            let found = page.find(id);
            if found.is_some() { return found; }
        }
        None
    }

}

impl Page {
    pub fn new(id: PipelineId, subpage_id: Option<SubpageId>,
           layout_chan: LayoutChan,
           window_size: WindowSizeData,
           resource_task: ResourceTask,
           constellation_chan: ConstellationChan,
           js_context: Rc<Cx>) -> Page {
        let js_info = JSPageInfo {
            dom_static: GlobalStaticData(),
            js_context: Untraceable::new(js_context),
        };
        let layout_rpc: Box<LayoutRPC> = {
            let (rpc_send, rpc_recv) = channel();
            let LayoutChan(ref lchan) = layout_chan;
            lchan.send(GetRPCMsg(rpc_send));
            rpc_recv.recv()
        };
        Page {
            id: id,
            subpage_id: subpage_id,
            frame: Traceable::new(RefCell::new(None)),
            layout_chan: Untraceable::new(layout_chan),
            layout_rpc: Untraceable::new(layout_rpc),
            layout_join_port: Untraceable::new(RefCell::new(None)),
            damage: Traceable::new(RefCell::new(None)),
            window_size: Traceable::new(Cell::new(window_size)),
            js_info: Traceable::new(RefCell::new(Some(js_info))),
            url: Untraceable::new(RefCell::new(None)),
            next_subpage_id: Traceable::new(Cell::new(SubpageId(0))),
            resize_event: Untraceable::new(Cell::new(None)),
            fragment_name: RefCell::new(None),
            last_reflow_id: Traceable::new(Cell::new(0)),
            resource_task: Untraceable::new(resource_task),
            constellation_chan: Untraceable::new(constellation_chan),
            children: Traceable::new(RefCell::new(vec!())),
            pending_reflows: Cell::new(0),
            avoided_reflows: Cell::new(0),
        }
    }

    pub fn flush_layout(&self, goal: ReflowGoal) {
        let damaged = self.damage.borrow().is_some();
        if damaged {
            let frame = self.frame();
            let window = frame.as_ref().unwrap().window.root();
            self.reflow(goal, window.control_chan.clone(), &**window.compositor);
        } else {
            self.avoided_reflows.set(self.avoided_reflows.get() + 1);
        }
    }

    pub fn layout(&self) -> &LayoutRPC {
        // FIXME This should probably be ReflowForQuery, not Display. All queries currently
        // currently rely on the display list, which means we can't destroy it by
        // doing a query reflow.
        self.flush_layout(ReflowForDisplay);
        self.join_layout(); //FIXME: is this necessary, or is layout_rpc's mutex good enough?
        let layout_rpc: &LayoutRPC = &**self.layout_rpc;
        layout_rpc
    }

    // must handle root case separately
    pub fn remove(&self, id: PipelineId) -> Option<Rc<Page>> {
        let remove_idx = {
            self.children
                .deref()
                .borrow_mut()
                .iter_mut()
                .enumerate()
                .find(|&(_idx, ref page_tree)| {
                    // FIXME: page_tree has a lifetime such that it's unusable for anything.
                    let page_tree_id = page_tree.id;
                    page_tree_id == id
                })
                .map(|(idx, _)| idx)
        };
        match remove_idx {
            Some(idx) => return Some(self.children.deref().borrow_mut().remove(idx).unwrap()),
            None => {
                for page_tree in self.children.deref().borrow_mut().iter_mut() {
                    match page_tree.remove(id) {
                        found @ Some(_) => return found,
                        None => (), // keep going...
                    }
                }
            }
        }
        None
    }
}

impl Iterator<Rc<Page>> for PageIterator {
    fn next(&mut self) -> Option<Rc<Page>> {
        if !self.stack.is_empty() {
            let next = self.stack.pop().unwrap();
            for child in next.children.deref().borrow().iter() {
                self.stack.push(child.clone());
            }
            Some(next.clone())
        } else {
            None
        }
    }
}

impl Page {
    pub fn mut_js_info<'a>(&'a self) -> RefMut<'a, Option<JSPageInfo>> {
        self.js_info.deref().borrow_mut()
    }

    pub fn js_info<'a>(&'a self) -> Ref<'a, Option<JSPageInfo>> {
        self.js_info.deref().borrow()
    }

    pub fn url<'a>(&'a self) -> Ref<'a, Option<(Url, bool)>> {
        self.url.deref().borrow()
    }

    pub fn mut_url<'a>(&'a self) -> RefMut<'a, Option<(Url, bool)>> {
        self.url.deref().borrow_mut()
    }

    pub fn frame<'a>(&'a self) -> Ref<'a, Option<Frame>> {
        self.frame.deref().borrow()
    }

    pub fn mut_frame<'a>(&'a self) -> RefMut<'a, Option<Frame>> {
        self.frame.deref().borrow_mut()
    }

    pub fn get_next_subpage_id(&self) -> SubpageId {
        let subpage_id = self.next_subpage_id.deref().get();
        let SubpageId(id_num) = subpage_id;
        self.next_subpage_id.deref().set(SubpageId(id_num + 1));
        subpage_id
    }

    /// Adds the given damage.
    pub fn damage(&self, level: DocumentDamageLevel) {
        let root = match *self.frame() {
            None => return,
            Some(ref frame) => frame.document.root().GetDocumentElement()
        };
        match root.root() {
            None => {},
            Some(root) => {
                let root: JSRef<Node> = NodeCast::from_ref(*root);
                let mut damage = *self.damage.deref().borrow_mut();
                match damage {
                    None => {}
                    Some(ref mut damage) => {
                        // FIXME(pcwalton): This is wrong. We should trace up to the nearest ancestor.
                        damage.root = root.to_trusted_node_address();
                        damage.level.add(level);
                        return
                    }
                }

                *self.damage.deref().borrow_mut() = Some(DocumentDamage {
                    root: root.to_trusted_node_address(),
                    level: level,
                })
            }
        };
    }

    pub fn get_url(&self) -> Url {
        self.url().as_ref().unwrap().ref0().clone()
    }

    // FIXME(cgaebel): join_layout is racey. What if the compositor triggers a
    // reflow between the "join complete" message and returning from this
    // function?

    /// Sends a ping to layout and waits for the response. The response will arrive when the
    /// layout task has finished any pending request messages.
    pub fn join_layout(&self) {
        let mut layout_join_port = self.layout_join_port.deref().borrow_mut();
        if layout_join_port.is_some() {
            let join_port = replace(&mut *layout_join_port, None);
            match join_port {
                Some(ref join_port) => {
                    match join_port.try_recv() {
                        Err(Empty) => {
                            info!("script: waiting on layout");
                            join_port.recv();
                        }
                        Ok(_) => {}
                        Err(Disconnected) => {
                            fail!("Layout task failed while script was waiting for a result.");
                        }
                    }

                    debug!("script: layout joined")
                }
                None => fail!("reader forked but no join port?"),
            }
        }
    }

    /// Reflows the page if it's possible to do so. This method will wait until the layout task has
    /// completed its current action, join the layout task, and then request a new layout run. It
    /// won't wait for the new layout computation to finish.
    ///
    /// If there is no window size yet, the page is presumed invisible and no reflow is performed.
    ///
    /// This function fails if there is no root frame.
    pub fn reflow(&self,
                  goal: ReflowGoal,
                  script_chan: ScriptControlChan,
                  compositor: &ScriptListener) {

        let root = match *self.frame() {
            None => return,
            Some(ref frame) => {
                frame.document.root().GetDocumentElement()
            }
        };

        match root.root() {
            None => {},
            Some(root) => {
                debug!("avoided {:d} reflows", self.avoided_reflows.get());
                self.avoided_reflows.set(0);

                debug!("script: performing reflow for goal {:?}", goal);

                // Now, join the layout so that they will see the latest changes we have made.
                self.join_layout();

                // Tell the user that we're performing layout.
                compositor.set_ready_state(self.id, PerformingLayout);

                // Layout will let us know when it's done.
                let (join_chan, join_port) = channel();
                let mut layout_join_port = self.layout_join_port.deref().borrow_mut();
                *layout_join_port = Some(join_port);

                let last_reflow_id = self.last_reflow_id.deref();
                last_reflow_id.set(last_reflow_id.get() + 1);

                let root: JSRef<Node> = NodeCast::from_ref(*root);
                let mut damage = self.damage.deref().borrow_mut();
                let window_size = self.window_size.deref().get();

                // Send new document and relevant styles to layout.
                let reflow = box Reflow {
                    document_root: root.to_trusted_node_address(),
                    url: self.get_url(),
                    iframe: self.subpage_id.is_some(),
                    goal: goal,
                    window_size: window_size,
                    script_chan: script_chan,
                    script_join_chan: join_chan,
                    damage: replace(&mut *damage, None).unwrap(),
                    id: last_reflow_id.get(),
                };

                let LayoutChan(ref chan) = *self.layout_chan;
                chan.send(ReflowMsg(reflow));

                debug!("script: layout forked")
            }
        }
    }

    /// Attempt to find a named element in this page's document.
    pub fn find_fragment_node(&self, fragid: DOMString) -> Option<Temporary<Element>> {
        let document = self.frame().as_ref().unwrap().document.root();
        document.find_fragment_node(fragid)
    }

    pub fn hit_test(&self, point: &Point2D<f32>) -> Option<UntrustedNodeAddress> {
        let frame = self.frame();
        let document = frame.as_ref().unwrap().document.root();
        let root = document.deref().GetDocumentElement().root();
        if root.is_none() {
            return None;
        }
        let root = root.unwrap();
        let root: JSRef<Node> = NodeCast::from_ref(*root);
        let address = match self.layout().hit_test(root.to_trusted_node_address(), *point) {
            Ok(HitTestResponse(node_address)) => {
                Some(node_address)
            }
            Err(()) => {
                debug!("layout query error");
                None
            }
        };
        address
    }

    pub fn get_nodes_under_mouse(&self, point: &Point2D<f32>) -> Option<Vec<UntrustedNodeAddress>> {
        let frame = self.frame();
        let document = frame.as_ref().unwrap().document.root();
        let root = document.deref().GetDocumentElement().root();
        if root.is_none() {
            return None;
        }
        let root = root.unwrap();
        let root: JSRef<Node> = NodeCast::from_ref(*root);
        let address = match self.layout().mouse_over(root.to_trusted_node_address(), *point) {
            Ok(MouseOverResponse(node_address)) => {
                Some(node_address)
            }
            Err(()) => {
                None
            }
        };
        address
    }
}

/// Information for one frame in the browsing context.
#[jstraceable]
#[must_root]
pub struct Frame {
    /// The document for this frame.
    pub document: JS<Document>,
    /// The window object for this frame.
    pub window: JS<Window>,
}

/// Encapsulation of the javascript information associated with each frame.
#[jstraceable]
pub struct JSPageInfo {
    /// Global static data related to the DOM.
    pub dom_static: GlobalStaticData,
    /// The JavaScript context.
    pub js_context: Untraceable<Rc<Cx>>,
}
