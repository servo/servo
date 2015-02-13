/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable};
use dom::bindings::utils::GlobalStaticData;
use dom::document::{Document, DocumentHelpers};
use dom::element::Element;
use dom::node::{Node, NodeHelpers};
use dom::window::Window;
use devtools_traits::DevtoolsControlChan;
use layout_interface::{
    ContentBoxResponse, ContentBoxesResponse,
    HitTestResponse, LayoutChan, LayoutRPC, MouseOverResponse, Msg, Reflow,
    ReflowGoal, ReflowQueryType,
    TrustedNodeAddress
};
use script_traits::{UntrustedNodeAddress, ScriptControlChan};

use geom::{Point2D, Rect, Size2D};
use js::rust::Cx;
use msg::compositor_msg::ScriptListener;
use msg::constellation_msg::{ConstellationChan, WindowSizeData};
use msg::constellation_msg::{PipelineId, SubpageId};
use net::resource_task::ResourceTask;
use net::storage_task::StorageTask;
use util::geometry::{Au, MAX_RECT};
use util::geometry;
use util::str::DOMString;
use util::smallvec::SmallVec;
use std::cell::{Cell, Ref, RefMut};
use std::sync::mpsc::{channel, Receiver};
use std::sync::mpsc::TryRecvError::{Empty, Disconnected};
use std::mem::replace;
use std::num::Float;
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
    pub last_reflow_id: Cell<uint>,

    /// The outermost frame containing the document, window, and page URL.
    pub frame: DOMRefCell<Option<Frame>>,

    /// A handle for communicating messages to the layout task.
    pub layout_chan: LayoutChan,

    /// A handle to perform RPC calls into the layout, quickly.
    layout_rpc: Box<LayoutRPC+'static>,

    /// The port that we will use to join layout. If this is `None`, then layout is not running.
    pub layout_join_port: DOMRefCell<Option<Receiver<()>>>,

    /// The current size of the window, in pixels.
    pub window_size: Cell<WindowSizeData>,

    js_info: DOMRefCell<Option<JSPageInfo>>,

    /// Cached copy of the most recent url loaded by the script
    /// TODO(tkuehn): this currently does not follow any particular caching policy
    /// and simply caches pages forever (!). The bool indicates if reflow is required
    /// when reloading.
    url: DOMRefCell<Option<(Url, bool)>>,

    next_subpage_id: Cell<SubpageId>,

    /// Pending resize event, if any.
    pub resize_event: Cell<Option<WindowSizeData>>,

    /// Pending scroll to fragment event, if any
    pub fragment_name: DOMRefCell<Option<String>>,

    /// Associated resource task for use by DOM objects like XMLHttpRequest
    pub resource_task: ResourceTask,

    /// A handle for communicating messages to the storage task.
    pub storage_task: StorageTask,

    /// A handle for communicating messages to the constellation task.
    pub constellation_chan: ConstellationChan,

    // Child Pages.
    pub children: DOMRefCell<Vec<Rc<Page>>>,

    /// An enlarged rectangle around the page contents visible in the viewport, used
    /// to prevent creating display list items for content that is far away from the viewport.
    pub page_clip_rect: Cell<Rect<Au>>,

    /// A flag to indicate whether the developer tools have requested live updates of
    /// page changes.
    pub devtools_wants_updates: Cell<bool>,

    /// For providing instructions to an optional devtools server.
    pub devtools_chan: Option<DevtoolsControlChan>,
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
        for page in self.children.borrow().iter() {
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
               storage_task: StorageTask,
               constellation_chan: ConstellationChan,
               js_context: Rc<Cx>,
               devtools_chan: Option<DevtoolsControlChan>) -> Page {
        let js_info = JSPageInfo {
            dom_static: GlobalStaticData::new(),
            js_context: js_context,
        };
        let layout_rpc: Box<LayoutRPC> = {
            let (rpc_send, rpc_recv) = channel();
            let LayoutChan(ref lchan) = layout_chan;
            lchan.send(Msg::GetRPC(rpc_send)).unwrap();
            rpc_recv.recv().unwrap()
        };
        Page {
            id: id,
            subpage_id: subpage_id,
            frame: DOMRefCell::new(None),
            layout_chan: layout_chan,
            layout_rpc: layout_rpc,
            layout_join_port: DOMRefCell::new(None),
            window_size: Cell::new(window_size),
            js_info: DOMRefCell::new(Some(js_info)),
            url: DOMRefCell::new(None),
            next_subpage_id: Cell::new(SubpageId(0)),
            resize_event: Cell::new(None),
            fragment_name: DOMRefCell::new(None),
            last_reflow_id: Cell::new(0),
            resource_task: resource_task,
            storage_task: storage_task,
            constellation_chan: constellation_chan,
            children: DOMRefCell::new(vec!()),
            page_clip_rect: Cell::new(MAX_RECT),
            devtools_wants_updates: Cell::new(false),
            devtools_chan: devtools_chan,
        }
    }

    pub fn flush_layout(&self, goal: ReflowGoal, query: ReflowQueryType) {
        let frame = self.frame();
        let window = frame.as_ref().unwrap().window.root();
        self.reflow(goal, window.r().control_chan().clone(), &mut **window.r().compositor(), query);
    }

    pub fn layout(&self) -> &LayoutRPC {
        &*self.layout_rpc
    }

    pub fn content_box_query(&self, content_box_request: TrustedNodeAddress) -> Rect<Au> {
        self.flush_layout(ReflowGoal::ForScriptQuery, ReflowQueryType::ContentBoxQuery(content_box_request));
        self.join_layout(); //FIXME: is this necessary, or is layout_rpc's mutex good enough?
        let ContentBoxResponse(rect) = self.layout_rpc.content_box();
        rect
    }

    pub fn content_boxes_query(&self, content_boxes_request: TrustedNodeAddress) -> Vec<Rect<Au>> {
        self.flush_layout(ReflowGoal::ForScriptQuery, ReflowQueryType::ContentBoxesQuery(content_boxes_request));
        self.join_layout(); //FIXME: is this necessary, or is layout_rpc's mutex good enough?
        let ContentBoxesResponse(rects) = self.layout_rpc.content_boxes();
        rects
    }

    // must handle root case separately
    pub fn remove(&self, id: PipelineId) -> Option<Rc<Page>> {
        let remove_idx = {
            self.children
                .borrow_mut()
                .iter_mut()
                .position(|page_tree| page_tree.id == id)
        };
        match remove_idx {
            Some(idx) => Some(self.children.borrow_mut().remove(idx)),
            None => {
                self.children
                    .borrow_mut()
                    .iter_mut()
                    .filter_map(|page_tree| page_tree.remove(id))
                    .next()
            }
        }
    }

    pub fn set_page_clip_rect_with_new_viewport(&self, viewport: Rect<f32>) -> bool {
        // We use a clipping rectangle that is five times the size of the of the viewport,
        // so that we don't collect display list items for areas too far outside the viewport,
        // but also don't trigger reflows every time the viewport changes.
        static VIEWPORT_EXPANSION: f32 = 2.0; // 2 lengths on each side plus original length is 5 total.
        let proposed_clip_rect = geometry::f32_rect_to_au_rect(
            viewport.inflate(viewport.size.width * VIEWPORT_EXPANSION,
            viewport.size.height * VIEWPORT_EXPANSION));
        let clip_rect = self.page_clip_rect.get();
        if proposed_clip_rect == clip_rect {
            return false;
        }

        let had_clip_rect = clip_rect != MAX_RECT;
        if had_clip_rect && !should_move_clip_rect(clip_rect, viewport) {
            return false;
        }

        self.page_clip_rect.set(proposed_clip_rect);

        // If we didn't have a clip rect, the previous display doesn't need rebuilding
        // because it was built for infinite clip (MAX_RECT).
        had_clip_rect
    }

    pub fn send_title_to_compositor(&self) {
        match *self.frame() {
            None => {}
            Some(ref frame) => {
                let window = frame.window.root();
                let document = frame.document.root();
                window.r().compositor().set_title(self.id, Some(document.r().Title()));
            }
        }
    }

    pub fn dirty_all_nodes(&self) {
        match *self.frame.borrow() {
            None => {}
            Some(ref frame) => frame.document.root().r().dirty_all_nodes(),
        }
    }
}

impl Iterator for PageIterator {
    type Item = Rc<Page>;

    fn next(&mut self) -> Option<Rc<Page>> {
        match self.stack.pop() {
            Some(next) => {
                for child in next.children.borrow().iter() {
                    self.stack.push(child.clone());
                }
                Some(next)
            },
            None => None,
        }
    }
}

impl Page {
    pub fn mut_js_info<'a>(&'a self) -> RefMut<'a, Option<JSPageInfo>> {
        self.js_info.borrow_mut()
    }

    pub unsafe fn unsafe_mut_js_info<'a>(&'a self) -> &'a mut Option<JSPageInfo> {
        self.js_info.borrow_for_script_deallocation()
    }

    pub fn js_info<'a>(&'a self) -> Ref<'a, Option<JSPageInfo>> {
        self.js_info.borrow()
    }

    pub fn url<'a>(&'a self) -> Ref<'a, Option<(Url, bool)>> {
        self.url.borrow()
    }

    pub fn mut_url<'a>(&'a self) -> RefMut<'a, Option<(Url, bool)>> {
        self.url.borrow_mut()
    }

    pub fn frame<'a>(&'a self) -> Ref<'a, Option<Frame>> {
        self.frame.borrow()
    }

    pub fn mut_frame<'a>(&'a self) -> RefMut<'a, Option<Frame>> {
        self.frame.borrow_mut()
    }

    pub fn get_next_subpage_id(&self) -> SubpageId {
        let subpage_id = self.next_subpage_id.get();
        let SubpageId(id_num) = subpage_id;
        self.next_subpage_id.set(SubpageId(id_num + 1));
        subpage_id
    }

    pub fn get_url(&self) -> Url {
        self.url().as_ref().unwrap().0.clone()
    }

    // FIXME(cgaebel): join_layout is racey. What if the compositor triggers a
    // reflow between the "join complete" message and returning from this
    // function?

    /// Sends a ping to layout and waits for the response. The response will arrive when the
    /// layout task has finished any pending request messages.
    fn join_layout(&self) {
        let mut layout_join_port = self.layout_join_port.borrow_mut();
        if let Some(join_port) = replace(&mut *layout_join_port, None) {
            match join_port.try_recv() {
                Err(Empty) => {
                    info!("script: waiting on layout");
                    join_port.recv().unwrap();
                }
                Ok(_) => {}
                Err(Disconnected) => {
                    panic!("Layout task failed while script was waiting for a result.");
                }
            }

            debug!("script: layout joined")
        }
    }

    /// Reflows the page if it's possible to do so and the page is dirty. This method will wait
    /// for the layout thread to complete (but see the `TODO` below). If there is no window size
    /// yet, the page is presumed invisible and no reflow is performed.
    ///
    /// TODO(pcwalton): Only wait for style recalc, since we have off-main-thread layout.
    pub fn reflow(&self,
                  goal: ReflowGoal,
                  script_chan: ScriptControlChan,
                  _: &mut ScriptListener,
                  query_type: ReflowQueryType) {
        let root = match *self.frame() {
            None => return,
            Some(ref frame) => {
                frame.document.root().r().GetDocumentElement()
            }
        };

        let root = match root.root() {
            None => return,
            Some(root) => root,
        };

        debug!("script: performing reflow for goal {:?}", goal);

        let root: JSRef<Node> = NodeCast::from_ref(root.r());
        if !root.get_has_dirty_descendants() {
            debug!("root has no dirty descendants; avoiding reflow");
            return
        }

        debug!("script: performing reflow for goal {:?}", goal);

        // Layout will let us know when it's done.
        let (join_chan, join_port) = channel();

        {
            let mut layout_join_port = self.layout_join_port.borrow_mut();
            *layout_join_port = Some(join_port);
        }

        let last_reflow_id = &self.last_reflow_id;
        last_reflow_id.set(last_reflow_id.get() + 1);

        let window_size = self.window_size.get();

        // Send new document and relevant styles to layout.
        let reflow = box Reflow {
            document_root: root.to_trusted_node_address(),
            url: self.get_url(),
            iframe: self.subpage_id.is_some(),
            goal: goal,
            window_size: window_size,
            script_chan: script_chan,
            script_join_chan: join_chan,
            id: last_reflow_id.get(),
            query_type: query_type,
            page_clip_rect: self.page_clip_rect.get(),
        };

        let LayoutChan(ref chan) = self.layout_chan;
        chan.send(Msg::Reflow(reflow)).unwrap();

        debug!("script: layout forked");

        self.join_layout();
    }

    /// Attempt to find a named element in this page's document.
    pub fn find_fragment_node(&self, fragid: DOMString) -> Option<Temporary<Element>> {
        let document = self.frame().as_ref().unwrap().document.root();
        document.r().find_fragment_node(fragid)
    }

    pub fn hit_test(&self, point: &Point2D<f32>) -> Option<UntrustedNodeAddress> {
        let frame = self.frame();
        let document = frame.as_ref().unwrap().document.root();
        let root = match document.r().GetDocumentElement().root() {
            None => return None,
            Some(root) => root,
        };
        let root: JSRef<Node> = NodeCast::from_ref(root.r());
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
        let root = match document.r().GetDocumentElement().root() {
            None => return None,
            Some(root) => root,
        };
        let root: JSRef<Node> = NodeCast::from_ref(root.r());
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

fn should_move_clip_rect(clip_rect: Rect<Au>, new_viewport: Rect<f32>) -> bool{
    let clip_rect = Rect(Point2D(geometry::to_frac_px(clip_rect.origin.x) as f32,
                                 geometry::to_frac_px(clip_rect.origin.y) as f32),
                         Size2D(geometry::to_frac_px(clip_rect.size.width) as f32,
                                geometry::to_frac_px(clip_rect.size.height) as f32));

    // We only need to move the clip rect if the viewport is getting near the edge of
    // our preexisting clip rect. We use half of the size of the viewport as a heuristic
    // for "close."
    static VIEWPORT_SCROLL_MARGIN_SIZE: f32 = 0.5;
    let viewport_scroll_margin = new_viewport.size * VIEWPORT_SCROLL_MARGIN_SIZE;

    (clip_rect.origin.x - new_viewport.origin.x).abs() <= viewport_scroll_margin.width ||
    (clip_rect.max_x() - new_viewport.max_x()).abs() <= viewport_scroll_margin.width ||
    (clip_rect.origin.y - new_viewport.origin.y).abs() <= viewport_scroll_margin.height ||
    (clip_rect.max_y() - new_viewport.max_y()).abs() <= viewport_scroll_margin.height
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
    pub js_context: Rc<Cx>,
}
