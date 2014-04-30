/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The script task is the task that owns the DOM in memory, runs JavaScript, and spawns parsing
//! and layout tasks.

use dom::attr::AttrMethods;
use dom::bindings::codegen::RegisterBindings;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, NodeCast, ElementCast, EventCast};
use dom::bindings::js::{JS, JSRef, RootCollection, Temporary, OptionalSettable};
use dom::bindings::js::OptionalRootable;
use dom::bindings::trace::{Traceable, Untraceable};
use dom::bindings::utils::{Reflectable, GlobalStaticData, wrap_for_same_compartment};
use dom::document::{Document, HTMLDocument, DocumentMethods, DocumentHelpers};
use dom::element::{Element, AttributeHandlers};
use dom::event::{Event_, ResizeEvent, ReflowEvent, ClickEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use dom::event::{Event, EventMethods};
use dom::uievent::{UIEvent, UIEventMethods};
use dom::eventtarget::{EventTarget, EventTargetHelpers};
use dom::node;
use dom::node::{Node, NodeHelpers};
use dom::window::{TimerId, Window};
use html::hubbub_html_parser::HtmlParserResult;
use html::hubbub_html_parser::{HtmlDiscoveredStyle, HtmlDiscoveredIFrame, HtmlDiscoveredScript};
use html::hubbub_html_parser;
use layout_interface::{AddStylesheetMsg, DocumentDamage};
use layout_interface::{DocumentDamageLevel, HitTestQuery, HitTestResponse, LayoutQuery, MouseOverQuery, MouseOverResponse};
use layout_interface::{LayoutChan, MatchSelectorsDocumentDamage, QueryMsg};
use layout_interface::{Reflow, ReflowDocumentDamage, ReflowForDisplay, ReflowGoal, ReflowMsg};
use layout_interface::ContentChangedDocumentDamage;
use layout_interface::UntrustedNodeAddress;
use layout_interface;

use geom::point::Point2D;
use geom::size::Size2D;
use js::global::DEBUG_FNS;
use js::jsapi::{JSObject, JS_CallFunctionValue, JS_DefineFunctions};
use js::jsapi::{JS_SetWrapObjectCallbacks, JS_SetGCZeal, JS_DEFAULT_ZEAL_FREQ};
use js::jsval::NullValue;
use js::rust::{Cx, RtUtils};
use js;
use servo_msg::compositor_msg::{FinishedLoading, LayerId, Loading, PerformingLayout};
use servo_msg::compositor_msg::{ScriptListener};
use servo_msg::constellation_msg::{ConstellationChan, IFrameSandboxed, IFrameUnsandboxed};
use servo_msg::constellation_msg::{LoadIframeUrlMsg, LoadCompleteMsg, LoadUrlMsg, NavigationDirection};
use servo_msg::constellation_msg::{PipelineId, SubpageId, Failure, FailureMsg};
use servo_msg::constellation_msg;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::geometry::to_frac_px;
use servo_util::url::parse_url;
use servo_util::task::send_on_failure;
use servo_util::namespace::Null;
use std::cast;
use std::cell::{RefCell, Ref, RefMut};
use std::comm::{channel, Sender, Receiver, Empty, Disconnected, Data};
use std::local_data;
use std::mem::replace;
use std::ptr;
use std::rc::Rc;
use std::task;
use url::Url;

use serialize::{Encoder, Encodable};

local_data_key!(pub StackRoots: *RootCollection)

/// Messages used to control the script task.
pub enum ScriptMsg {
    /// Loads a new URL on the specified pipeline.
    LoadMsg(PipelineId, Url),
    /// Gives a channel and ID to a layout task, as well as the ID of that layout's parent
    AttachLayoutMsg(NewLayoutInfo),
    /// Instructs the script task to send a navigate message to the constellation.
    NavigateMsg(NavigationDirection),
    /// Sends a DOM event.
    SendEventMsg(PipelineId, Event_),
    /// Window resized.  Sends a DOM event eventually, but first we combine events.
    ResizeMsg(PipelineId, Size2D<uint>),
    /// Fires a JavaScript timeout.
    FireTimerMsg(PipelineId, TimerId),
    /// Notifies script that reflow is finished.
    ReflowCompleteMsg(PipelineId, uint),
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactiveMsg(PipelineId, Size2D<uint>),
    /// Notifies the script that a pipeline should be closed.
    ExitPipelineMsg(PipelineId),
    /// Notifies the script that a window associated with a particular pipeline should be closed.
    ExitWindowMsg(PipelineId),
}

pub struct NewLayoutInfo {
    pub old_id: PipelineId,
    pub new_id: PipelineId,
    pub layout_chan: LayoutChan,
}

/// Encapsulates external communication with the script task.
#[deriving(Clone)]
pub struct ScriptChan(pub Sender<ScriptMsg>);

impl<S: Encoder<E>, E> Encodable<S, E> for ScriptChan {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}

impl ScriptChan {
    /// Creates a new script chan.
    pub fn new() -> (Receiver<ScriptMsg>, ScriptChan) {
        let (chan, port) = channel();
        (port, ScriptChan(chan))
    }
}

/// Encapsulates a handle to a frame and its associated layout information.
#[deriving(Encodable)]
pub struct Page {
    /// Pipeline id associated with this page.
    pub id: PipelineId,

    /// Unique id for last reflow request; used for confirming completion reply.
    pub last_reflow_id: Traceable<RefCell<uint>>,

    /// The outermost frame containing the document, window, and page URL.
    pub frame: Traceable<RefCell<Option<Frame>>>,

    /// A handle for communicating messages to the layout task.
    pub layout_chan: Untraceable<LayoutChan>,

    /// The port that we will use to join layout. If this is `None`, then layout is not running.
    pub layout_join_port: Untraceable<RefCell<Option<Receiver<()>>>>,

    /// What parts of the document are dirty, if any.
    pub damage: Traceable<RefCell<Option<DocumentDamage>>>,

    /// The current size of the window, in pixels.
    pub window_size: Untraceable<RefCell<Size2D<uint>>>,

    pub js_info: Traceable<RefCell<Option<JSPageInfo>>>,

    /// Cached copy of the most recent url loaded by the script
    /// TODO(tkuehn): this currently does not follow any particular caching policy
    /// and simply caches pages forever (!). The bool indicates if reflow is required
    /// when reloading.
    pub url: Untraceable<RefCell<Option<(Url, bool)>>>,

    pub next_subpage_id: Untraceable<RefCell<SubpageId>>,

    /// Pending resize event, if any.
    pub resize_event: Untraceable<RefCell<Option<Size2D<uint>>>>,

    /// Pending scroll to fragment event, if any
    pub fragment_node: Traceable<RefCell<Option<JS<Element>>>>
}

pub struct PageTree {
    pub page: Rc<Page>,
    pub inner: Vec<PageTree>,
}

pub struct PageTreeIterator<'a> {
    stack: Vec<&'a mut PageTree>,
}

impl PageTree {
    fn new(id: PipelineId, layout_chan: LayoutChan, window_size: Size2D<uint>) -> PageTree {
        PageTree {
            page: Rc::new(Page {
                id: id,
                frame: Traceable::new(RefCell::new(None)),
                layout_chan: Untraceable::new(layout_chan),
                layout_join_port: Untraceable::new(RefCell::new(None)),
                damage: Traceable::new(RefCell::new(None)),
                window_size: Untraceable::new(RefCell::new(window_size)),
                js_info: Traceable::new(RefCell::new(None)),
                url: Untraceable::new(RefCell::new(None)),
                next_subpage_id: Untraceable::new(RefCell::new(SubpageId(0))),
                resize_event: Untraceable::new(RefCell::new(None)),
                fragment_node: Traceable::new(RefCell::new(None)),
                last_reflow_id: Traceable::new(RefCell::new(0)),
            }),
            inner: vec!(),
        }
    }

    fn id(&self) -> PipelineId {
        self.page().id
    }

    fn page<'a>(&'a self) -> &'a Page {
        &*self.page
    }

    pub fn find<'a> (&'a mut self, id: PipelineId) -> Option<&'a mut PageTree> {
        if self.page().id == id { return Some(self); }
        for page_tree in self.inner.mut_iter() {
            let found = page_tree.find(id);
            if found.is_some() { return found; }
        }
        None
    }

    pub fn iter<'a>(&'a mut self) -> PageTreeIterator<'a> {
        PageTreeIterator {
            stack: vec!(self),
        }
    }

    // must handle root case separately
    pub fn remove(&mut self, id: PipelineId) -> Option<PageTree> {
        let remove_idx = {
            self.inner.mut_iter()
                .enumerate()
                .find(|&(_idx, ref page_tree)| {
                    // FIXME: page_tree has a lifetime such that it's unusable for anything.
                    let page_tree = unsafe {
                        cast::transmute_region(page_tree)
                    };
                    page_tree.id() == id
                })
                .map(|(idx, _)| idx)
        };
        match remove_idx {
            Some(idx) => return Some(self.inner.remove(idx).unwrap()),
            None => {
                for page_tree in self.inner.mut_iter() {
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

impl<'a> Iterator<Rc<Page>> for PageTreeIterator<'a> {
    fn next(&mut self) -> Option<Rc<Page>> {
        if !self.stack.is_empty() {
            let next = self.stack.pop().unwrap();
            for child in next.inner.mut_iter() {
                self.stack.push(child);
            }
            Some(next.page.clone())
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

    /// Adds the given damage.
    pub fn damage(&self, level: DocumentDamageLevel) {
        let root = match *self.frame() {
            None => return,
            Some(ref frame) => frame.document.root().GetDocumentElement()
        };
        match root.root() {
            None => {},
            Some(root) => {
                let root: &JSRef<Node> = NodeCast::from_ref(&*root);
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
        self.url().get_ref().ref0().clone()
    }

    /// Sends a ping to layout and waits for the response. The response will arrive when the
    /// layout task has finished any pending request messages.
    pub fn join_layout(&self) {
        let mut layout_join_port = self.layout_join_port.deref().borrow_mut();
        if layout_join_port.is_some() {
            let join_port = replace(&mut *layout_join_port, None);
            match join_port {
                Some(ref join_port) => {
                    match join_port.try_recv() {
                        Empty => {
                            info!("script: waiting on layout");
                            join_port.recv();
                        }
                        Data(_) => {}
                        Disconnected => {
                            fail!("Layout task failed while script was waiting for a result.");
                        }
                    }

                    debug!("script: layout joined")
                }
                None => fail!("reader forked but no join port?"),
            }
        }
    }

    /// Sends the given query to layout.
    pub fn query_layout<T: Send>(&self,
                                 query: LayoutQuery,
                                 response_port: Receiver<T>)
                                 -> T {
        self.join_layout();
        let LayoutChan(ref chan) = *self.layout_chan;
        chan.send(QueryMsg(query));
        response_port.recv()
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
                  script_chan: ScriptChan,
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
                debug!("script: performing reflow for goal {:?}", goal);

                // Now, join the layout so that they will see the latest changes we have made.
                self.join_layout();

                // Tell the user that we're performing layout.
                compositor.set_ready_state(PerformingLayout);

                // Layout will let us know when it's done.
                let (join_chan, join_port) = channel();
                let mut layout_join_port = self.layout_join_port.deref().borrow_mut();
                *layout_join_port = Some(join_port);

                let mut last_reflow_id = self.last_reflow_id.deref().borrow_mut();
                *last_reflow_id += 1;

                let root: &JSRef<Node> = NodeCast::from_ref(&*root);
                let mut damage = self.damage.deref().borrow_mut();
                let window_size = self.window_size.deref().borrow();

                // Send new document and relevant styles to layout.
                let reflow = ~Reflow {
                    document_root: root.to_trusted_node_address(),
                    url: self.get_url(),
                    goal: goal,
                    window_size: *window_size,
                    script_chan: script_chan,
                    script_join_chan: join_chan,
                    damage: replace(&mut *damage, None).unwrap(),
                    id: *last_reflow_id,
                };

                let LayoutChan(ref chan) = *self.layout_chan;
                chan.send(ReflowMsg(reflow));

                debug!("script: layout forked")
            }
        }
    }

    fn find_fragment_node(&self, fragid: ~str) -> Option<Temporary<Element>> {
        let document = self.frame().get_ref().document.root();
        match document.deref().GetElementById(fragid.to_owned()) {
            Some(node) => Some(node),
            None => {
                let doc_node: &JSRef<Node> = NodeCast::from_ref(&*document);
                let mut anchors = doc_node.traverse_preorder()
                                          .filter(|node| node.is_anchor_element());
                anchors.find(|node| {
                    let elem: &JSRef<Element> = ElementCast::to_ref(node).unwrap();
                    elem.get_attribute(Null, "name").root().map_or(false, |attr| {
                        attr.deref().value_ref() == fragid
                    })
                }).map(|node| Temporary::from_rooted(ElementCast::to_ref(&node).unwrap()))
            }
        }
    }

    pub fn initialize_js_info(&self, js_context: Rc<Cx>, global: *JSObject) {
        assert!(global.is_not_null());

        // Note that the order that these variables are initialized is _not_ arbitrary. Switching
        // them around can -- and likely will -- lead to things breaking.

        unsafe {
            JS_SetGCZeal(js_context.deref().ptr, 0, JS_DEFAULT_ZEAL_FREQ);
        }

        js_context.set_default_options_and_version();
        js_context.set_logging_error_reporter();

        let mut js_info = self.mut_js_info();
        *js_info = Some(JSPageInfo {
            dom_static: GlobalStaticData(),
            js_context: Untraceable::new(js_context),
        });
    }

    pub fn hit_test(&self, point: &Point2D<f32>) -> Option<UntrustedNodeAddress> {
        let frame = self.frame();
        let document = frame.get_ref().document.root();
        let root = document.deref().GetDocumentElement().root();
        if root.is_none() {
            return None;
        }
        let root = root.unwrap();
        let root: &JSRef<Node> = NodeCast::from_ref(&*root);
        let (chan, port) = channel();
        let address = match self.query_layout(HitTestQuery(root.to_trusted_node_address(), *point, chan), port) {
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
        let document = frame.get_ref().document.root();
        let root = document.deref().GetDocumentElement().root();
        if root.is_none() {
            return None;
        }
        let root = root.unwrap();
        let root: &JSRef<Node> = NodeCast::from_ref(&*root);
        let (chan, port) = channel();
        let address = match self.query_layout(MouseOverQuery(root.to_trusted_node_address(), *point, chan), port) {
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
#[deriving(Encodable)]
pub struct Frame {
    /// The document for this frame.
    pub document: JS<Document>,
    /// The window object for this frame.
    pub window: JS<Window>,
}

/// Encapsulation of the javascript information associated with each frame.
#[deriving(Encodable)]
pub struct JSPageInfo {
    /// Global static data related to the DOM.
    pub dom_static: GlobalStaticData,
    /// The JavaScript context.
    pub js_context: Untraceable<Rc<Cx>>,
}

struct StackRootTLS;

impl StackRootTLS {
    fn new(roots: &RootCollection) -> StackRootTLS {
        local_data::set(StackRoots, roots as *RootCollection);
        StackRootTLS
    }
}

impl Drop for StackRootTLS {
    fn drop(&mut self) {
        let _ = local_data::pop(StackRoots);
    }
}

/// Information for an entire page. Pages are top-level browsing contexts and can contain multiple
/// frames.
///
/// FIXME: Rename to `Page`, following WebKit?
pub struct ScriptTask {
    /// A handle to the information pertaining to page layout
    pub page_tree: RefCell<PageTree>,
    /// A handle to the image cache task.
    pub image_cache_task: ImageCacheTask,
    /// A handle to the resource task.
    pub resource_task: ResourceTask,

    /// The port on which the script task receives messages (load URL, exit, etc.)
    pub port: Receiver<ScriptMsg>,
    /// A channel to hand out when some other task needs to be able to respond to a message from
    /// the script task.
    pub chan: ScriptChan,

    /// For communicating load url messages to the constellation
    pub constellation_chan: ConstellationChan,
    /// A handle to the compositor for communicating ready state messages.
    pub compositor: ~ScriptListener,

    /// The JavaScript runtime.
    pub js_runtime: js::rust::rt,

    pub mouse_over_targets: RefCell<Option<Vec<JS<Node>>>>
}

/// In the event of task failure, all data on the stack runs its destructor. However, there
/// are no reachable, owning pointers to the DOM memory, so it never gets freed by default
/// when the script task fails. The ScriptMemoryFailsafe uses the destructor bomb pattern
/// to forcibly tear down the JS compartments for pages associated with the failing ScriptTask.
struct ScriptMemoryFailsafe<'a> {
    owner: Option<&'a ScriptTask>,
}

impl<'a> ScriptMemoryFailsafe<'a> {
    fn neuter(&mut self) {
        self.owner = None;
    }

    fn new(owner: &'a ScriptTask) -> ScriptMemoryFailsafe<'a> {
        ScriptMemoryFailsafe {
            owner: Some(owner),
        }
    }
}

#[unsafe_destructor]
impl<'a> Drop for ScriptMemoryFailsafe<'a> {
    fn drop(&mut self) {
        match self.owner {
            Some(owner) => {
                let mut page_tree = owner.page_tree.borrow_mut();
                for page in page_tree.iter() {
                    *page.mut_js_info() = None;
                }
            }
            None => (),
        }
    }
}

impl ScriptTask {
    /// Creates a new script task.
    pub fn new(id: PipelineId,
               compositor: ~ScriptListener,
               layout_chan: LayoutChan,
               port: Receiver<ScriptMsg>,
               chan: ScriptChan,
               constellation_chan: ConstellationChan,
               resource_task: ResourceTask,
               img_cache_task: ImageCacheTask,
               window_size: Size2D<uint>)
               -> Rc<ScriptTask> {
        let js_runtime = js::rust::rt();

        unsafe {
            JS_SetWrapObjectCallbacks(js_runtime.deref().ptr,
                                      ptr::null(),
                                      wrap_for_same_compartment,
                                      ptr::null());
        }

        Rc::new(ScriptTask {
            page_tree: RefCell::new(PageTree::new(id, layout_chan, window_size)),

            image_cache_task: img_cache_task,
            resource_task: resource_task,

            port: port,
            chan: chan,
            constellation_chan: constellation_chan,
            compositor: compositor,

            js_runtime: js_runtime,
            mouse_over_targets: RefCell::new(None)
        })
    }

    /// Starts the script task. After calling this method, the script task will loop receiving
    /// messages on its port.
    pub fn start(&self) {
        while self.handle_msgs() {
            // Go on...
        }
    }

    pub fn create<C:ScriptListener + Send>(
                  id: PipelineId,
                  compositor: ~C,
                  layout_chan: LayoutChan,
                  port: Receiver<ScriptMsg>,
                  chan: ScriptChan,
                  constellation_chan: ConstellationChan,
                  failure_msg: Failure,
                  resource_task: ResourceTask,
                  image_cache_task: ImageCacheTask,
                  window_size: Size2D<uint>) {
        let mut builder = task::task().named("ScriptTask");
        let ConstellationChan(const_chan) = constellation_chan.clone();
        send_on_failure(&mut builder, FailureMsg(failure_msg), const_chan);
        builder.spawn(proc() {
            let script_task = ScriptTask::new(id,
                                              compositor as ~ScriptListener,
                                              layout_chan,
                                              port,
                                              chan,
                                              constellation_chan,
                                              resource_task,
                                              image_cache_task,
                                              window_size);
            let mut failsafe = ScriptMemoryFailsafe::new(&*script_task);
            script_task.start();

            // This must always be the very last operation performed before the task completes
            failsafe.neuter();
        });
    }

    /// Handle incoming control messages.
    fn handle_msgs(&self) -> bool {
        let roots = RootCollection::new();
        let _stack_roots_tls = StackRootTLS::new(&roots);

        // Handle pending resize events.
        // Gather them first to avoid a double mut borrow on self.
        let mut resizes = vec!();

        {
            let mut page_tree = self.page_tree.borrow_mut();
            for page in page_tree.iter() {
                // Only process a resize if layout is idle.
                let layout_join_port = page.layout_join_port.deref().borrow();
                if layout_join_port.is_none() {
                    let mut resize_event = page.resize_event.deref().borrow_mut();
                    match resize_event.take() {
                        Some(size) => resizes.push((page.id, size)),
                        None => ()
                    }
                }
            }
        }

        for (id, Size2D { width, height }) in resizes.move_iter() {
            self.handle_event(id, ResizeEvent(width, height));
        }

        // Store new resizes, and gather all other events.
        let mut sequential = vec!();

        // Receive at least one message so we don't spinloop.
        let mut event = self.port.recv();

        loop {
            match event {
                ResizeMsg(id, size) => {
                    let mut page_tree = self.page_tree.borrow_mut();
                    let page = page_tree.find(id).expect("resize sent to nonexistent pipeline").page();
                    let mut resize_event = page.resize_event.deref().borrow_mut();
                    *resize_event = Some(size);
                }
                _ => {
                    sequential.push(event);
                }
            }

            match self.port.try_recv() {
                Empty | Disconnected => break,
                Data(ev) => event = ev,
            }
        }

        // Process the gathered events.
        for msg in sequential.move_iter() {
            match msg {
                // TODO(tkuehn) need to handle auxiliary layouts for iframes
                AttachLayoutMsg(new_layout_info) => self.handle_new_layout(new_layout_info),
                LoadMsg(id, url) => self.load(id, url),
                SendEventMsg(id, event) => self.handle_event(id, event),
                FireTimerMsg(id, timer_id) => self.handle_fire_timer_msg(id, timer_id),
                NavigateMsg(direction) => self.handle_navigate_msg(direction),
                ReflowCompleteMsg(id, reflow_id) => self.handle_reflow_complete_msg(id, reflow_id),
                ResizeInactiveMsg(id, new_size) => self.handle_resize_inactive_msg(id, new_size),
                ExitPipelineMsg(id) => if self.handle_exit_pipeline_msg(id) { return false },
                ExitWindowMsg(id) => self.handle_exit_window_msg(id),
                ResizeMsg(..) => fail!("should have handled ResizeMsg already"),
            }
        }

        true
    }

    fn handle_new_layout(&self, new_layout_info: NewLayoutInfo) {
        debug!("Script: new layout: {:?}", new_layout_info);
        let NewLayoutInfo {
            old_id,
            new_id,
            layout_chan
        } = new_layout_info;

        let mut page_tree = self.page_tree.borrow_mut();
        let parent_page_tree = page_tree.find(old_id).expect("ScriptTask: received a layout
            whose parent has a PipelineId which does not correspond to a pipeline in the script
            task's page tree. This is a bug.");
        let new_page_tree = {
            let window_size = parent_page_tree.page().window_size.deref().borrow();
            PageTree::new(new_id, layout_chan, *window_size)
        };
        parent_page_tree.inner.push(new_page_tree);
    }

    /// Handles a timer that fired.
    fn handle_fire_timer_msg(&self, id: PipelineId, timer_id: TimerId) {
        let mut page_tree = self.page_tree.borrow_mut();
        let page = page_tree.find(id).expect("ScriptTask: received fire timer msg for a
            pipeline ID not associated with this script task. This is a bug.").page();
        let frame = page.frame();
        let mut window = frame.get_ref().window.root();

        let this_value = window.deref().reflector().get_jsobject();

        let is_interval;
        match window.deref().active_timers.find(&timer_id) {
            None => return,
            Some(timer_handle) => {
                // TODO: Support extra arguments. This requires passing a `*JSVal` array as `argv`.
                let rval = NullValue();
                let js_info = page.js_info();
                let cx = js_info.get_ref().js_context.deref().deref().ptr;
                unsafe {
                    JS_CallFunctionValue(cx, this_value, *timer_handle.data.funval,
                                         0, ptr::null(), &rval);
                }

                is_interval = timer_handle.data.is_interval;
            }
        }

        if !is_interval {
            window.deref_mut().active_timers.remove(&timer_id);
        }
    }

    /// Handles a notification that reflow completed.
    fn handle_reflow_complete_msg(&self, pipeline_id: PipelineId, reflow_id: uint) {
        debug!("Script: Reflow {:?} complete for {:?}", reflow_id, pipeline_id);
        let mut page_tree = self.page_tree.borrow_mut();
        let page = page_tree.find(pipeline_id).expect(
            "ScriptTask: received a load message for a layout channel that is not associated \
             with this script task. This is a bug.").page();
        let last_reflow_id = page.last_reflow_id.deref().borrow();
        if *last_reflow_id == reflow_id {
            let mut layout_join_port = page.layout_join_port.deref().borrow_mut();
            *layout_join_port = None;
        }
        self.compositor.set_ready_state(FinishedLoading);
    }

    /// Handles a navigate forward or backward message.
    /// TODO(tkuehn): is it ever possible to navigate only on a subframe?
    fn handle_navigate_msg(&self, direction: NavigationDirection) {
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(constellation_msg::NavigateMsg(direction));
    }

    /// Window was resized, but this script was not active, so don't reflow yet
    fn handle_resize_inactive_msg(&self, id: PipelineId, new_size: Size2D<uint>) {
        let mut page_tree = self.page_tree.borrow_mut();
        let page = page_tree.find(id).expect("Received resize message for PipelineId not associated
            with a page in the page tree. This is a bug.").page();
        let mut window_size = page.window_size.deref().borrow_mut();
        *window_size = new_size;
        let mut page_url = page.mut_url();
        let last_loaded_url = replace(&mut *page_url, None);
        for url in last_loaded_url.iter() {
            *page_url = Some((url.ref0().clone(), true));
        }
    }

    /// We have gotten a window.close from script, which we pass on to the compositor.
    /// We do not shut down the script task now, because the compositor will ask the
    /// constellation to shut down the pipeline, which will clean everything up
    /// normally. If we do exit, we will tear down the DOM nodes, possibly at a point
    /// where layout is still accessing them.
    fn handle_exit_window_msg(&self, _: PipelineId) {
        debug!("script task handling exit window msg");

        // TODO(tkuehn): currently there is only one window,
        // so this can afford to be naive and just shut down the
        // compositor. In the future it'll need to be smarter.
        self.compositor.close();
    }

    /// Handles a request to exit the script task and shut down layout.
    /// Returns true if the script task should shut down and false otherwise.
    fn handle_exit_pipeline_msg(&self, id: PipelineId) -> bool {
        // If root is being exited, shut down all pages
        let mut page_tree = self.page_tree.borrow_mut();
        if page_tree.page().id == id {
            for page in page_tree.iter() {
                debug!("shutting down layout for root page {:?}", page.id);
                shut_down_layout(&*page)
            }
            return true
        }

        // otherwise find just the matching page and exit all sub-pages
        match page_tree.remove(id) {
            Some(ref mut page_tree) => {
                for page in page_tree.iter() {
                    debug!("shutting down layout for page {:?}", page.id);
                    shut_down_layout(&*page)
                }
                false
            }
            // TODO(tkuehn): pipeline closing is currently duplicated across
            // script and constellation, which can cause this to happen. Constellation
            // needs to be smarter about exiting pipelines.
            None => false,
        }

    }

    /// The entry point to document loading. Defines bindings, sets up the window and document
    /// objects, parses HTML and CSS, and kicks off initial layout.
    fn load(&self, pipeline_id: PipelineId, url: Url) {
        debug!("ScriptTask: loading {:?} on page {:?}", url, pipeline_id);

        let mut page_tree = self.page_tree.borrow_mut();
        let page_tree = page_tree.find(pipeline_id).expect("ScriptTask: received a load
            message for a layout channel that is not associated with this script task. This
            is a bug.");
        let page = page_tree.page();

        let last_loaded_url = replace(&mut *page.mut_url(), None);
        for loaded in last_loaded_url.iter() {
            let (ref loaded, needs_reflow) = *loaded;
            if *loaded == url {
                *page.mut_url() = Some((loaded.clone(), false));
                if needs_reflow {
                    page.damage(ContentChangedDocumentDamage);
                    page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor);
                }
                return;
            }
        }

        let cx = self.js_runtime.cx();
        // Create the window and document objects.
        let mut window = Window::new(cx.deref().ptr,
                                     page_tree.page.clone(),
                                     self.chan.clone(),
                                     self.compositor.dup(),
                                     self.image_cache_task.clone()).root();
        page.initialize_js_info(cx.clone(), window.reflector().get_jsobject());
        let mut document = Document::new(&*window, Some(url.clone()), HTMLDocument, None).root();
        window.deref_mut().init_browser_context(&*document);

        {
            let mut js_info = page.mut_js_info();
            RegisterBindings::Register(&window.unrooted(), js_info.get_mut_ref());
        }

        self.compositor.set_ready_state(Loading);
        // Parse HTML.
        //
        // Note: We can parse the next document in parallel with any previous documents.
        let html_parsing_result = hubbub_html_parser::parse_html(page,
                                                                 &mut *document,
                                                                 url.clone(),
                                                                 self.resource_task.clone());

        let HtmlParserResult {
            discovery_port
        } = html_parsing_result;

        {
            // Create the root frame.
            let mut frame = page.mut_frame();
            *frame = Some(Frame {
                document: document.deref().unrooted(),
                window: window.deref().unrooted(),
            });
        }

        // Send style sheets over to layout.
        //
        // FIXME: These should be streamed to layout as they're parsed. We don't need to stop here
        // in the script task.

        let mut js_scripts = None;
        loop {
            match discovery_port.recv_opt() {
                Some(HtmlDiscoveredScript(scripts)) => {
                    assert!(js_scripts.is_none());
                    js_scripts = Some(scripts);
                }
                Some(HtmlDiscoveredStyle(sheet)) => {
                    let LayoutChan(ref chan) = *page.layout_chan;
                    chan.send(AddStylesheetMsg(sheet));
                }
                Some(HtmlDiscoveredIFrame((iframe_url, subpage_id, sandboxed))) => {
                    let SubpageId(num) = subpage_id;
                    *page.next_subpage_id.deref().borrow_mut() = SubpageId(num + 1);
                    let sandboxed = if sandboxed {
                        IFrameSandboxed
                    } else {
                        IFrameUnsandboxed
                    };
                    let ConstellationChan(ref chan) = self.constellation_chan;
                    chan.send(LoadIframeUrlMsg(iframe_url,
                                               pipeline_id,
                                               subpage_id,
                                               sandboxed));
                }
                None => break
            }
        }

        // Kick off the initial reflow of the page.
        document.content_changed();

        let fragment = url.fragment.as_ref().map(|ref fragment| fragment.to_owned());

        {
            // No more reflow required
            let mut page_url = page.mut_url();
            *page_url = Some((url.clone(), false));
        }

        // Receive the JavaScript scripts.
        assert!(js_scripts.is_some());
        let js_scripts = js_scripts.take_unwrap();
        debug!("js_scripts: {:?}", js_scripts);

        // Define debug functions.
        unsafe {
            assert!(JS_DefineFunctions((*cx).ptr,
                                       window.reflector().get_jsobject(),
                                       DEBUG_FNS.as_ptr()) != 0);
        }

        // Evaluate every script in the document.
        for file in js_scripts.iter() {
            let global_obj = window.reflector().get_jsobject();
            //FIXME: this should have some kind of error handling, or explicitly
            //       drop an exception on the floor.
            match cx.evaluate_script(global_obj, file.data.clone(), file.url.to_str(), 1) {
                Ok(_) => (),
                Err(_) => println!("evaluate_script failed")
            }
        }

        // We have no concept of a document loader right now, so just dispatch the
        // "load" event as soon as we've finished executing all scripts parsed during
        // the initial load.
        let mut event = Event::new(&*window).root();
        event.InitEvent("load".to_owned(), false, false);
        let doctarget: &JSRef<EventTarget> = EventTargetCast::from_ref(&*document);
        let wintarget: &JSRef<EventTarget> = EventTargetCast::from_ref(&*window);
        let _ = wintarget.dispatch_event_with_target(Some((*doctarget).clone()),
                                                     &mut *event);

        let mut fragment_node = page.fragment_node.deref().borrow_mut();
        (*fragment_node).assign(fragment.map_or(None, |fragid| page.find_fragment_node(fragid)));

        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(LoadCompleteMsg(page.id, url));
    }

    fn scroll_fragment_point(&self, pipeline_id: PipelineId, node: &JSRef<Element>) {
        let node: &JSRef<Node> = NodeCast::from_ref(node);
        let rect = node.get_bounding_content_box();
        let point = Point2D(to_frac_px(rect.origin.x).to_f32().unwrap(),
                            to_frac_px(rect.origin.y).to_f32().unwrap());
        // FIXME(#2003, pcwalton): This is pretty bogus when multiple layers are involved.
        // Really what needs to happen is that this needs to go through layout to ask which
        // layer the element belongs to, and have it send the scroll message to the
        // compositor.
        self.compositor.scroll_fragment_point(pipeline_id, LayerId::null(), point);
    }

    /// This is the main entry point for receiving and dispatching DOM events.
    ///
    /// TODO: Actually perform DOM event dispatch.
    fn handle_event(&self, pipeline_id: PipelineId, event: Event_) {
        fn get_page<'a>(page_tree: &'a mut PageTree, pipeline_id: PipelineId) -> &'a Page {
            page_tree.find(pipeline_id).expect("ScriptTask: received an event \
                message for a layout channel that is not associated with this script task.\
                This is a bug.").page()
        }

        match event {
            ResizeEvent(new_width, new_height) => {
                debug!("script got resize event: {:u}, {:u}", new_width, new_height);

                let window = {
                    let mut page_tree = self.page_tree.borrow_mut();
                    let page = get_page(&mut *page_tree, pipeline_id);
                    {
                        let mut window_size = page.window_size.deref().borrow_mut();
                        *window_size = Size2D(new_width, new_height);
                    }

                    let frame = page.frame();
                    if frame.is_some() {
                        page.damage(ReflowDocumentDamage);
                        page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor)
                    }

                    let mut fragment_node = page.fragment_node.deref().borrow_mut();
                    match fragment_node.take().map(|node| node.root()) {
                        Some(node) => self.scroll_fragment_point(pipeline_id, &*node),
                        None => {}
                    }

                    frame.as_ref().map(|frame| Temporary::new(frame.window.clone()))
                };

                match window.root() {
                    Some(mut window) => {
                        // http://dev.w3.org/csswg/cssom-view/#resizing-viewports
                        // https://dvcs.w3.org/hg/dom3events/raw-file/tip/html/DOM3-Events.html#event-type-resize
                        let mut uievent = UIEvent::new(&*window).root();
                        uievent.InitUIEvent("resize".to_owned(), false, false,
                                            Some((*window).clone()), 0i32);
                        let event: &mut JSRef<Event> = EventCast::from_mut_ref(&mut *uievent);

                        let wintarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(&mut *window);
                        let _ = wintarget.dispatch_event_with_target(None, &mut *event);
                    }
                    None => ()
                }
            }

            // FIXME(pcwalton): This reflows the entire document and is not incremental-y.
            ReflowEvent => {
                debug!("script got reflow event");

                let mut page_tree = self.page_tree.borrow_mut();
                let page = get_page(&mut *page_tree, pipeline_id);
                let frame = page.frame();
                if frame.is_some() {
                    page.damage(MatchSelectorsDocumentDamage);
                    page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor)
                }
            }

            ClickEvent(_button, point) => {
                debug!("ClickEvent: clicked at {:?}", point);
                let mut page_tree = self.page_tree.borrow_mut();
                let page = get_page(&mut *page_tree, pipeline_id);
                match page.hit_test(&point) {
                    Some(node_address) => {
                        debug!("node address is {:?}", node_address);
                        let mut node =
                            node::from_untrusted_node_address(self.js_runtime.deref().ptr,
                                                              node_address).root();
                        debug!("clicked on {:s}", node.deref().debug_str());

                        // Traverse node generations until a node that is an element is
                        // found.
                        while !node.deref().is_element() {
                            match node.deref().parent_node() {
                                Some(parent) => node = parent.root(),
                                None => break,
                            }
                        }

                        if node.deref().is_element() {
                            let element: &JSRef<Element> = ElementCast::to_ref(&*node).unwrap();
                            if "a" == element.deref().local_name {
                                self.load_url_from_element(page, element)
                            }
                        }
                    }

                    None => {}
                }
            }
            MouseDownEvent(..) => {}
            MouseUpEvent(..) => {}
            MouseMoveEvent(point) => {
                let mut page_tree = self.page_tree.borrow_mut();
                let page = get_page(&mut *page_tree, pipeline_id);
                match page.get_nodes_under_mouse(&point) {
                    Some(node_address) => {

                        let mut target_list = vec!();
                        let mut target_compare = false;

                        let mouse_over_targets = &mut *self.mouse_over_targets.borrow_mut();
                        match *mouse_over_targets {
                            Some(ref mut mouse_over_targets) => {
                                for node in mouse_over_targets.mut_iter() {
                                    let mut node = node.root();
                                    node.set_hover_state(false);
                                }
                            }
                            None => {}
                        }

                        for node_address in node_address.iter() {
                            let mut node =
                                node::from_untrusted_node_address(
                                    self.js_runtime.deref().ptr, *node_address).root();
                            // Traverse node generations until a node that is an element is
                            // found.
                            while !node.is_element() {
                                match node.parent_node() {
                                    Some(parent) => node = parent.root(),
                                    None => break,
                                }
                            }

                            if node.is_element() {
                                node.set_hover_state(true);

                                match *mouse_over_targets {
                                    Some(ref mouse_over_targets) => {
                                        if !target_compare {
                                            target_compare = !mouse_over_targets.contains(&node.unrooted());
                                        }
                                    }
                                    None => {}
                                }
                                target_list.push(node.unrooted());
                            }
                        }
                        match *mouse_over_targets {
                            Some(ref mouse_over_targets) => {
                                if mouse_over_targets.len() != target_list.len() {
                                    target_compare = true;
                                }
                            }
                            None => { target_compare = true; }
                        }

                        if target_compare {
                            if mouse_over_targets.is_some() {
                                page.damage(MatchSelectorsDocumentDamage);
                                page.reflow(ReflowForDisplay, self.chan.clone(), self.compositor);
                            }
                            *mouse_over_targets = Some(target_list);
                        }
                    }

                    None => {}
              }
            }
        }
    }

    fn load_url_from_element(&self, page: &Page, element: &JSRef<Element>) {
        // if the node's element is "a," load url from href attr
        let attr = element.get_attribute(Null, "href");
        for href in attr.root().iter() {
            debug!("ScriptTask: clicked on link to {:s}", href.Value());
            let click_frag = href.deref().value_ref().starts_with("#");
            let base_url = Some(page.get_url());
            debug!("ScriptTask: current url is {:?}", base_url);
            let url = parse_url(href.deref().value_ref(), base_url);

            if click_frag {
                match page.find_fragment_node(url.fragment.unwrap()).root() {
                    Some(node) => self.scroll_fragment_point(page.id, &*node),
                    None => {}
                }
            } else {
                let ConstellationChan(ref chan) = self.constellation_chan;
                chan.send(LoadUrlMsg(page.id, url));
            }
        }
    }
}

/// Shuts down layout for the given page.
fn shut_down_layout(page: &Page) {
    page.join_layout();

    // Tell the layout task to begin shutting down.
    let (response_chan, response_port) = channel();
    let LayoutChan(ref chan) = *page.layout_chan;
    chan.send(layout_interface::PrepareToExitMsg(response_chan));
    response_port.recv();

    // Destroy all nodes. Setting frame and js_info to None will trigger our
    // compartment to shutdown, run GC, etc.

    let mut js_info = page.mut_js_info();

    let mut frame = page.mut_frame();
    *frame = None;
    *js_info = None;

    // Destroy the layout task. If there were node leaks, layout will now crash safely.
    chan.send(layout_interface::ExitNowMsg);
}
