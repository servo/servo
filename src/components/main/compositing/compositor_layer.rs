/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::quadtree::{Quadtree, Normal, Hidden};
use pipeline::CompositionPipeline;
use windowing::{MouseWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent};
use windowing::{MouseWindowMouseUpEvent};

use azure::azure_hl::Color;
use geom::matrix::identity;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::render_task::{ReRenderMsg, UnusedBufferMsg};
use gfx;
use layers::layers::{ContainerLayerKind, ContainerLayer, Flip, NoFlip, TextureLayer};
use layers::layers::TextureLayerKind;
use layers::platform::surface::{NativeCompositingGraphicsContext, NativeSurfaceMethods};
use layers::texturegl::{Texture, TextureTarget};
use script::dom::event::{ClickEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use script::script_task::{ScriptChan, SendEventMsg};
use servo_msg::compositor_msg::{Epoch, FixedPosition, LayerBuffer, LayerBufferSet, LayerId};
use servo_msg::compositor_msg::{ScrollPolicy, Tile};
use servo_msg::constellation_msg::PipelineId;
use std::rc::Rc;

#[cfg(target_os="macos")]
#[cfg(target_os="android")]
use layers::layers::VerticalFlip;
#[cfg(not(target_os="macos"))]
use layers::texturegl::TextureTarget2D;
#[cfg(target_os="macos")]
use layers::texturegl::TextureTargetRectangle;

/// The amount of memory usage allowed per layer.
static MAX_TILE_MEMORY_PER_LAYER: uint = 10000000;

/// The CompositorLayer represents an element on a page that has a unique scroll
/// or animation behavior. This can include absolute positioned elements, iframes, etc.
/// Each layer can also have child layers.
///
/// FIXME(#2003, pcwalton): This should be merged with the concept of a layer in `rust-layers` and
/// ultimately removed, except as a set of helper methods on `rust-layers` layers.
pub struct CompositorLayer {
    /// This layer's pipeline. BufferRequests and mouse events will be sent through this.
    pub pipeline: CompositionPipeline,

    /// The ID of this layer within the pipeline.
    pub id: LayerId,

    /// The bounds of this layer in terms of its parent (a.k.a. the scissor box).
    pub bounds: Rect<f32>,

    /// The size of the underlying page in page coordinates. This is an option
    /// because we may not know the size of the page until layout is finished completely.
    /// if we have no size yet, the layer is hidden until a size message is recieved.
    pub page_size: Option<Size2D<f32>>,

    /// The offset of the page due to scrolling. (0,0) is when the window sees the
    /// top left corner of the page.
    pub scroll_offset: Point2D<f32>,

    /// This layer's children. These could be iframes or any element which
    /// differs in scroll behavior from its parent. Each is associated with a
    /// ContainerLayer which determines its position relative to its parent and
    /// clipping rect. Children are stored in the order in which they are drawn.
    pub children: Vec<CompositorLayerChild>,

    /// This layer's quadtree. This is where all buffers are stored for this layer.
    pub quadtree: MaybeQuadtree,

    /// The root layer of this CompositorLayer's layer tree. Buffers are collected
    /// from the quadtree and inserted here when the layer is painted to the screen.
    pub root_layer: Rc<ContainerLayer>,

    /// When set to true, this layer is ignored by its parents. This is useful for
    /// soft deletion or when waiting on a page size.
    pub hidden: bool,

    /// A monotonically increasing counter that keeps track of the current epoch.
    /// add_buffer() calls that don't match the current epoch will be ignored.
    pub epoch: Epoch,

    /// The behavior of this layer when a scroll message is received.
    pub wants_scroll_events: WantsScrollEventsFlag,

    /// Whether an ancestor layer that receives scroll events moves this layer.
    pub scroll_policy: ScrollPolicy,

    /// True if CPU rendering is enabled, false if we're using GPU rendering.
    pub cpu_painting: bool,

    /// The color to use for the unrendered-content void
    pub unrendered_color: Color,
}

/// Helper struct for keeping CompositorLayer children organized.
pub struct CompositorLayerChild {
    /// The child itself.
    pub child: ~CompositorLayer,
    /// A ContainerLayer managed by the parent node. This deals with clipping and
    /// positioning, and is added above the child's layer tree.
    pub container: Rc<ContainerLayer>,
}

/// Helper enum for storing quadtrees. Either contains a quadtree, or contains
/// information from which a quadtree can be built.
enum MaybeQuadtree {
    Tree(Quadtree<~LayerBuffer>),
    NoTree(uint, Option<uint>),
}

impl MaybeQuadtree {
    fn tile_size(&self) -> uint {
        match *self {
            Tree(ref quadtree) => quadtree.max_tile_size,
            NoTree(tile_size, _) => tile_size,
        }
    }
}

#[deriving(Eq, Clone)]
pub enum WantsScrollEventsFlag {
    WantsScrollEvents,
    DoesntWantScrollEvents,
}

fn create_container_layer_from_rect(rect: Rect<f32>) -> Rc<ContainerLayer> {
    let container = Rc::new(ContainerLayer());
    *container.scissor.borrow_mut() = Some(rect);
    container.common.borrow_mut().transform =
        identity().translate(rect.origin.x, rect.origin.y, 0f32);
    container
}

trait Clampable {
    fn clamp(&self, mn: &Self, mx: &Self) -> Self;
}

impl Clampable for f32 {
    /// Returns the number constrained within the range `mn <= self <= mx`.
    /// If any of the numbers are `NAN` then `NAN` is returned.
    #[inline]
    fn clamp(&self, mn: &f32, mx: &f32) -> f32 {
        match () {
            _ if self.is_nan()   => *self,
            _ if !(*self <= *mx) => *mx,
            _ if !(*self >= *mn) => *mn,
            _                    => *self,
        }
    }
}


impl CompositorLayer {
    /// Creates a new `CompositorLayer`.
    fn new(pipeline: CompositionPipeline,
           layer_id: LayerId,
           bounds: Rect<f32>,
           page_size: Option<Size2D<f32>>,
           tile_size: uint,
           cpu_painting: bool,
           wants_scroll_events: WantsScrollEventsFlag,
           scroll_policy: ScrollPolicy)
           -> CompositorLayer {
        CompositorLayer {
            pipeline: pipeline,
            id: layer_id,
            bounds: bounds,
            page_size: page_size,
            scroll_offset: Point2D(0f32, 0f32),
            children: vec!(),
            quadtree: match page_size {
                None => NoTree(tile_size, Some(MAX_TILE_MEMORY_PER_LAYER)),
                Some(page_size) => {
                    Tree(Quadtree::new(Size2D(page_size.width as uint, page_size.height as uint),
                                       tile_size,
                                       Some(MAX_TILE_MEMORY_PER_LAYER)))
                }
            },
            root_layer: Rc::new(ContainerLayer()),
            hidden: true,
            epoch: Epoch(0),
            wants_scroll_events: wants_scroll_events,
            scroll_policy: scroll_policy,
            cpu_painting: cpu_painting,
            unrendered_color: gfx::color::rgba(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Creates a new root `CompositorLayer` bound to a composition pipeline with an optional page
    /// size. If no page size is given, the layer is initially hidden and initialized without a
    /// quadtree.
    pub fn new_root(pipeline: CompositionPipeline,
                    page_size: Size2D<f32>,
                    tile_size: uint,
                    cpu_painting: bool)
                    -> CompositorLayer {
        CompositorLayer {
            pipeline: pipeline,
            id: LayerId::null(),
            bounds: Rect(Point2D(0f32, 0f32), page_size),
            page_size: Some(page_size),
            scroll_offset: Point2D(0f32, 0f32),
            children: vec!(),
            quadtree: NoTree(tile_size, Some(MAX_TILE_MEMORY_PER_LAYER)),
            root_layer: Rc::new(ContainerLayer()),
            hidden: false,
            epoch: Epoch(0),
            wants_scroll_events: WantsScrollEvents,
            scroll_policy: FixedPosition,
            cpu_painting: cpu_painting,
            unrendered_color: gfx::color::rgba(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Adds a child layer to the layer with the given ID and the given pipeline, if it doesn't
    /// exist yet. The child layer will have the same pipeline, tile size, memory limit, and CPU
    /// painting status as its parent.
    ///
    /// Returns:
    ///   * True if the layer was added;
    ///   * True if the layer was not added because it already existed;
    ///   * False if the layer could not be added because no suitable parent layer with the given
    ///     ID and pipeline could be found.
    pub fn add_child_if_necessary(&mut self,
                                  container_layer: Rc<ContainerLayer>,
                                  pipeline_id: PipelineId,
                                  parent_layer_id: LayerId,
                                  child_layer_id: LayerId,
                                  rect: Rect<f32>,
                                  page_size: Size2D<f32>,
                                  scroll_policy: ScrollPolicy)
                                  -> bool {
        if self.pipeline.id != pipeline_id || self.id != parent_layer_id {
            return self.children.mut_iter().any(|kid_holder| {
                kid_holder.child.add_child_if_necessary(kid_holder.container.clone(),
                                                        pipeline_id,
                                                        parent_layer_id,
                                                        child_layer_id,
                                                        rect,
                                                        page_size,
                                                        scroll_policy)
            })
        }

        // See if we've already made this child layer.
        if self.children.iter().any(|kid_holder| {
                    kid_holder.child.pipeline.id == pipeline_id &&
                    kid_holder.child.id == child_layer_id
                }) {
            return true
        }

        let mut kid = ~CompositorLayer::new(self.pipeline.clone(),
                                            child_layer_id,
                                            rect,
                                            Some(page_size),
                                            self.quadtree.tile_size(),
                                            self.cpu_painting,
                                            DoesntWantScrollEvents,
                                            scroll_policy);

        kid.hidden = false;

        // Place the kid's layer in a container...
        let kid_container = create_container_layer_from_rect(rect);
        ContainerLayer::add_child_start(kid_container.clone(),
                                        ContainerLayerKind(kid.root_layer.clone()));

        // ...and add *that* container as a child of the container passed in.
        ContainerLayer::add_child_end(container_layer,
                                      ContainerLayerKind(kid_container.clone()));

        self.children.push(CompositorLayerChild {
            child: kid,
            container: kid_container,
        });
        true
    }

    /// Move the layer's descendants that don't want scroll events and scroll by a relative
    /// specified amount in page coordinates. This also takes in a cursor position to see if the
    /// mouse is over child layers first. If a layer successfully scrolled, returns true; otherwise
    /// returns false, so a parent layer can scroll instead.
    pub fn handle_scroll_event(&mut self,
                               delta: Point2D<f32>,
                               cursor: Point2D<f32>,
                               window_size: Size2D<f32>)
                               -> bool {
        // If this layer is hidden, neither it nor its children will scroll.
        if self.hidden {
            return false
        }

        // If this layer doesn't want scroll events, neither it nor its children can handle scroll
        // events.
        if self.wants_scroll_events != WantsScrollEvents {
            return false
        }

        // Allow children to scroll.
        let cursor = cursor - self.scroll_offset;
        for child in self.children.mut_iter() {
            match *child.container.scissor.borrow() {
                None => {
                    error!("CompositorLayer: unable to perform cursor hit test for layer");
                }
                Some(rect) => {
                    if cursor.x >= rect.origin.x && cursor.x < rect.origin.x + rect.size.width
                        && cursor.y >= rect.origin.y && cursor.y < rect.origin.y + rect.size.height
                        && child.child.handle_scroll_event(delta,
                                                           cursor - rect.origin,
                                                           rect.size) {
                        return true
                    }
                }
            }
        }

        // This scroll event is mine!
        // Scroll this layer!
        let old_origin = self.scroll_offset;
        self.scroll_offset = self.scroll_offset + delta;

        // bounds checking
        let page_size = match self.page_size {
            Some(size) => size,
            None => fail!("CompositorLayer: tried to scroll with no page size set"),
        };
        let min_x = (window_size.width - page_size.width).min(0.0);
        self.scroll_offset.x = self.scroll_offset.x.clamp(&min_x, &0.0);
        let min_y = (window_size.height - page_size.height).min(0.0);
        self.scroll_offset.y = self.scroll_offset.y.clamp(&min_y, &0.0);

        if old_origin - self.scroll_offset == Point2D(0f32, 0f32) {
            return false
        }

        self.scroll(self.scroll_offset)
    }

    #[allow(dead_code)]
    fn dump_layer_tree(&self, layer: Rc<ContainerLayer>, indent: ~str) {
        println!("{}scissor {:?}", indent, layer.scissor.borrow());
        for kid in layer.children() {
            match kid {
                ContainerLayerKind(ref container_layer) => {
                    self.dump_layer_tree((*container_layer).clone(), indent + "  ");
                }
                TextureLayerKind(_) => {
                    println!("{}  (texture layer)", indent);
                }
            }
        }
    }

    /// Actually scrolls the descendants of a layer that scroll. This is called by
    /// `handle_scroll_event` above when it determines that a layer wants to scroll.
    fn scroll(&mut self, scroll_offset: Point2D<f32>) -> bool {
        let mut result = false;

        // Only scroll this layer if it's not fixed-positioned.
        if self.scroll_policy != FixedPosition {
            // Scroll this layer!
            self.scroll_offset = scroll_offset;

            self.root_layer.common.borrow_mut().set_transform(
                identity().translate(self.scroll_offset.x, self.scroll_offset.y, 0.0));

            result = true
        }

        for kid_holder in self.children.mut_iter() {
            result = kid_holder.child.scroll(scroll_offset) || result;
        }

        result
    }

    // Takes in a MouseWindowEvent, determines if it should be passed to children, and
    // sends the event off to the appropriate pipeline. NB: the cursor position is in
    // page coordinates.
    pub fn send_mouse_event(&self, event: MouseWindowEvent, cursor: Point2D<f32>) {
        let cursor = cursor - self.scroll_offset;
        for child in self.children.iter().filter(|&x| !x.child.hidden) {
            match *child.container.scissor.borrow() {
                None => {
                    error!("CompositorLayer: unable to perform cursor hit test for layer");
                }
                Some(rect) => {
                    if cursor.x >= rect.origin.x && cursor.x < rect.origin.x + rect.size.width
                        && cursor.y >= rect.origin.y && cursor.y < rect.origin.y + rect.size.height {
                        child.child.send_mouse_event(event, cursor - rect.origin);
                        return;
                    }
                }
            }
        }

        // This mouse event is mine!
        let message = match event {
            MouseWindowClickEvent(button, _) => ClickEvent(button, cursor),
            MouseWindowMouseDownEvent(button, _) => MouseDownEvent(button, cursor),
            MouseWindowMouseUpEvent(button, _) => MouseUpEvent(button, cursor),
        };
        let ScriptChan(ref chan) = self.pipeline.script_chan;
        chan.try_send(SendEventMsg(self.pipeline.id.clone(), message));
    }

    pub fn send_mouse_move_event(&self, cursor: Point2D<f32>) {
        let message = MouseMoveEvent(cursor);
        let ScriptChan(ref chan) = self.pipeline.script_chan;
        chan.try_send(SendEventMsg(self.pipeline.id.clone(), message));
    }

    // Given the current window size, determine which tiles need to be (re-)rendered and sends them
    // off the the appropriate renderer. Returns true if and only if the scene should be repainted.
    pub fn get_buffer_request(&mut self,
                              graphics_context: &NativeCompositingGraphicsContext,
                              window_rect: Rect<f32>,
                              scale: f32)
                              -> bool {
        let mut redisplay = false;
        match self.quadtree {
            NoTree(..) => {}
            Tree(ref mut quadtree) => {
                let (request, unused) = quadtree.get_tile_rects_page(window_rect, scale);

                // Workaround to make redisplay visible outside block.
                redisplay = !unused.is_empty();
                if redisplay {
                    // Send back unused tiles.
                    self.pipeline.render_chan.try_send(UnusedBufferMsg(unused));
                }
                if !request.is_empty() {
                    // Ask for tiles.
                    //
                    // FIXME(#2003, pcwalton): We may want to batch these up in the case in which
                    // one page has multiple layers, to avoid the user seeing inconsistent states.
                    let msg = ReRenderMsg(request, scale, self.id, self.epoch);
                    self.pipeline.render_chan.try_send(msg);
                }
            }
        };

        if redisplay {
            self.build_layer_tree(graphics_context);
        }

        let transform = |x: &mut CompositorLayerChild| -> bool {
            match *x.container.scissor.borrow() {
                Some(scissor) => {
                    let mut new_rect = window_rect;
                    new_rect.origin.x = new_rect.origin.x - x.child.scroll_offset.x;
                    new_rect.origin.y = new_rect.origin.y - x.child.scroll_offset.y;
                    match new_rect.intersection(&scissor) {
                        Some(new_rect) => {
                            // Child layers act as if they are rendered at (0,0), so we
                            // subtract the layer's (x,y) coords in its containing page
                            // to make the child_rect appear in coordinates local to it.
                            let child_rect = Rect(new_rect.origin.sub(&scissor.origin),
                                                  new_rect.size);
                            x.child.get_buffer_request(graphics_context, child_rect, scale)
                        }
                        None => {
                            false // Layer is offscreen
                        }
                    }
                }
                None => fail!("child layer not clipped!"),
            }
        };
        self.children.mut_iter().filter(|x| !x.child.hidden)
            .map(transform)
            .fold(false, |a, b| a || b) || redisplay
    }


    // Move the sublayer to an absolute position in page coordinates relative to its parent,
    // and clip the layer to the specified size in page coordinates.
    // If the layer is hidden and has a defined page size, unhide it.
    // This method returns false if the specified layer is not found.
    pub fn set_clipping_rect(&mut self,
                             pipeline_id: PipelineId,
                             layer_id: LayerId,
                             new_rect: Rect<f32>)
                             -> bool {
        debug!("compositor_layer: starting set_clipping_rect()");
        match self.children.iter().position(|kid_holder| {
                pipeline_id == kid_holder.child.pipeline.id &&
                layer_id == kid_holder.child.id
            }) {
            Some(i) => {
                debug!("compositor_layer: node found for set_clipping_rect()");
                let child_node = self.children.get_mut(i);
                child_node.container.common.borrow_mut().set_transform(
                    identity().translate(new_rect.origin.x, new_rect.origin.y, 0.0));
                let old_rect = child_node.container.scissor.borrow().clone();
                *child_node.container.scissor.borrow_mut() = Some(new_rect);
                match self.quadtree {
                    NoTree(..) => {} // Nothing to do
                        Tree(ref mut quadtree) => {
                        match old_rect {
                            Some(old_rect) => {
                                quadtree.set_status_page(old_rect, Normal, false); // Rect is unhidden
                            }
                            None => {} // Nothing to do
                        }
                        quadtree.set_status_page(new_rect, Hidden, false); // Hide the new rect
                    }
                }
                // If possible, unhide child
                if child_node.child.hidden && child_node.child.page_size.is_some() {
                    child_node.child.hidden = false;
                }
                true
            }
            None => {
                // ID does not match any of our immediate children, so recurse on
                // descendents (including hidden children)
                self.children
                    .mut_iter()
                    .map(|kid_holder| &mut kid_holder.child)
                    .any(|kid| kid.set_clipping_rect(pipeline_id, layer_id, new_rect))
            }
        }
    }

    // Set the layer's page size. This signals that the renderer is ready for BufferRequests.
    // If the layer is hidden and has a defined clipping rect, unhide it.
    // This method returns false if the specified layer is not found.
    pub fn resize(&mut self,
                  pipeline_id: PipelineId,
                  layer_id: LayerId,
                  new_size: Size2D<f32>,
                  window_size: Size2D<f32>,
                  epoch: Epoch)
                  -> bool {
        debug!("compositor_layer: starting resize()");
        if self.pipeline.id != pipeline_id || self.id != layer_id {
            return self.resize_helper(pipeline_id, layer_id, new_size, epoch)
        }

        debug!("compositor_layer: layer found for resize()");
        self.epoch = epoch;
        self.page_size = Some(new_size);
        match self.quadtree {
            Tree(ref mut quadtree) => {
                self.pipeline
                    .render_chan
                    .try_send(UnusedBufferMsg(quadtree.resize(new_size.width as uint,
                                                              new_size.height as uint)));
            }
            NoTree(tile_size, max_mem) => {
                self.quadtree = Tree(Quadtree::new(Size2D(new_size.width as uint,
                                                          new_size.height as uint),
                                                   tile_size,
                                                   max_mem))
            }
        }
        // Call scroll for bounds checking if the page shrunk. Use (-1, -1) as the cursor position
        // to make sure the scroll isn't propagated downwards.
        self.handle_scroll_event(Point2D(0f32, 0f32), Point2D(-1f32, -1f32), window_size);
        self.hidden = false;
        self.set_occlusions();
        true
    }

    pub fn move(&mut self,
                pipeline_id: PipelineId,
                layer_id: LayerId,
                origin: Point2D<f32>,
                window_size: Size2D<f32>)
                -> bool {
        // Search children for the right layer to move.
        if self.pipeline.id != pipeline_id || self.id != layer_id {
            return self.children.mut_iter().any(|kid_holder| {
                kid_holder.child.move(pipeline_id, layer_id, origin, window_size)
            })
        }

        if self.wants_scroll_events != WantsScrollEvents {
            return false
        }

        // Scroll this layer!
        let old_origin = self.scroll_offset;
        self.scroll_offset = Point2D(0f32, 0f32) - origin;

        // bounds checking
        let page_size = match self.page_size {
            Some(size) => size,
            None => fail!("CompositorLayer: tried to scroll with no page size set"),
        };
        let min_x = (window_size.width - page_size.width).min(0.0);
        self.scroll_offset.x = self.scroll_offset.x.clamp(&min_x, &0.0);
        let min_y = (window_size.height - page_size.height).min(0.0);
        self.scroll_offset.y = self.scroll_offset.y.clamp(&min_y, &0.0);

        // check to see if we scrolled
        if old_origin - self.scroll_offset == Point2D(0f32, 0f32) {
            return false;
        }

        self.scroll(self.scroll_offset)
    }

    // Returns whether the layer should be vertically flipped.
    #[cfg(target_os="macos")]
    fn texture_flip_and_target(cpu_painting: bool, size: Size2D<uint>) -> (Flip, TextureTarget) {
        let flip = if cpu_painting {
            NoFlip
        } else {
            VerticalFlip
        };

        (flip, TextureTargetRectangle(size))
    }

    #[cfg(target_os="android")]
    fn texture_flip_and_target(cpu_painting: bool, size: Size2D<uint>) -> (Flip, TextureTarget) {
        let flip = if cpu_painting {
            NoFlip
        } else {
            VerticalFlip
        };

        (flip, TextureTarget2D)
    }

    #[cfg(target_os="linux")]
    fn texture_flip_and_target(_: bool, _: Size2D<uint>) -> (Flip, TextureTarget) {
        (NoFlip, TextureTarget2D)
    }

    // A helper method to resize sublayers.
    fn resize_helper(&mut self,
                     pipeline_id: PipelineId,
                     layer_id: LayerId,
                     new_size: Size2D<f32>,
                     epoch: Epoch)
                     -> bool {
        debug!("compositor_layer: starting resize_helper()");
        let found = match self.children.iter().position(|kid_holder| {
                pipeline_id == kid_holder.child.pipeline.id &&
                layer_id == kid_holder.child.id
            }) {
            Some(i) => {
                debug!("compositor_layer: layer found for resize_helper()");
                let child_node = self.children.get_mut(i);
                let child = &mut child_node.child;
                child.epoch = epoch;
                child.page_size = Some(new_size);
                match child.quadtree {
                    Tree(ref mut quadtree) => {
                        child.pipeline.render_chan.try_send(UnusedBufferMsg(quadtree.resize(new_size.width as uint,
                                                                                            new_size.height as uint)));
                    }
                    NoTree(tile_size, max_mem) => {
                        child.quadtree = Tree(Quadtree::new(Size2D(new_size.width as uint,
                                                                   new_size.height as uint),
                                                            tile_size,
                                                            max_mem))
                    }
                }
                match *child_node.container.scissor.borrow() {
                    Some(scissor) => {
                        // Call scroll for bounds checking if the page shrunk. Use (-1, -1) as the
                        // cursor position to make sure the scroll isn't propagated downwards.
                        child.handle_scroll_event(Point2D(0f32, 0f32),
                                                  Point2D(-1f32, -1f32),
                                                  scissor.size);
                        child.hidden = false;
                    }
                    None => {} // Nothing to do
                }
                true
            }
            None => false,
        };

        if found { // Boolean flag to get around double borrow of self
            self.set_occlusions();
            return true
        }

        // If we got here, the layer's ID does not match ours, so recurse on descendents (including
        // hidden children).
        self.children
            .mut_iter()
            .map(|kid_holder| &mut kid_holder.child)
            .any(|kid_holder| kid_holder.resize_helper(pipeline_id, layer_id, new_size, epoch))
    }

    // Collect buffers from the quadtree. This method IS NOT recursive, so child CompositorLayers
    // are not rebuilt directly from this method.
    pub fn build_layer_tree(&mut self, graphics_context: &NativeCompositingGraphicsContext) {
        // Iterate over the children of the container layer.
        let mut current_layer_child = self.root_layer.first_child.borrow().clone();

        // Delete old layer.
        while current_layer_child.is_some() {
            let trash = current_layer_child.clone().unwrap();
            current_layer_child.clone().unwrap().with_common(|common| {
                current_layer_child = common.next_sibling.clone();
            });
            ContainerLayer::remove_child(self.root_layer.clone(), trash);
        }

        // Add new tiles.
        let quadtree = match self.quadtree {
            NoTree(..) => fail!("CompositorLayer: cannot build layer tree for {:?},
                               no quadtree initialized", self.pipeline.id),
            Tree(ref mut quadtree) => quadtree,
        };

        let all_tiles = quadtree.get_all_tiles();
        for buffer in all_tiles.iter() {
            debug!("osmain: compositing buffer rect {}", buffer.rect);

            let size = Size2D(buffer.screen_pos.size.width as int,
                              buffer.screen_pos.size.height as int);

            // Find or create a texture layer.
            let texture_layer;
            current_layer_child = match current_layer_child.clone() {
                None => {
                    debug!("osmain: adding new texture layer");

                    // Determine, in a platform-specific way, whether we should flip the texture
                    // and which target to use.
                    let (flip, target) =
                            CompositorLayer::texture_flip_and_target(self.cpu_painting,
                                                                     buffer.screen_pos.size);

                    // Make a new texture and bind the layer buffer's surface to it.
                    let texture = Texture::new(target);
                    debug!("COMPOSITOR binding to native surface {:d}",
                           buffer.native_surface.get_id() as int);
                    buffer.native_surface.bind_to_texture(graphics_context, &texture, size);

                    // Make a texture layer and add it.
                    texture_layer = Rc::new(TextureLayer::new(texture,
                                                              buffer.screen_pos.size,
                                                              flip));
                    ContainerLayer::add_child_end(self.root_layer.clone(),
                                                  TextureLayerKind(texture_layer.clone()));
                    None
                }
                Some(TextureLayerKind(existing_texture_layer)) => {
                    texture_layer = existing_texture_layer.clone();

                    let texture = &existing_texture_layer.texture;
                    buffer.native_surface.bind_to_texture(graphics_context, texture, size);

                    // Move on to the next sibling.
                    current_layer_child.unwrap().with_common(|common| {
                        common.next_sibling.clone()
                    })
                }
                Some(_) => fail!("found unexpected layer kind"),
            };

            // Set the layer's transform.
            let rect = buffer.rect;
            let transform = identity().translate(rect.origin.x, rect.origin.y, 0.0);
            let transform = transform.scale(rect.size.width, rect.size.height, 1.0);
            texture_layer.common.borrow_mut().set_transform(transform);
        }

        // Add child layers.
        for child in self.children.mut_iter().filter(|x| !x.child.hidden) {
            current_layer_child = match current_layer_child {
                None => {
                    {
                        let mut common = child.container.common.borrow_mut();
                        (*common).parent = None;
                        common.prev_sibling = None;
                        common.next_sibling = None;
                    }
                    ContainerLayer::add_child_end(self.root_layer.clone(),
                                                  ContainerLayerKind(child.container.clone()));
                    None
                }
                Some(_) => {
                    fail!("CompositorLayer: Layer tree failed to delete");
                }
            };
        }
    }

    // Add LayerBuffers to the specified layer. Returns the layer buffer set back if the layer that
    // matches the given pipeline ID was not found; otherwise returns None and consumes the layer
    // buffer set.
    //
    // If the epoch of the message does not match the layer's epoch, the message is ignored, the
    // layer buffer set is consumed, and None is returned.
    pub fn add_buffers(&mut self,
                       graphics_context: &NativeCompositingGraphicsContext,
                       pipeline_id: PipelineId,
                       layer_id: LayerId,
                       mut new_buffers: ~LayerBufferSet,
                       epoch: Epoch)
                       -> Option<~LayerBufferSet> {
        debug!("compositor_layer: starting add_buffers()");
        if self.pipeline.id != pipeline_id || self.id != layer_id {
            // ID does not match ours, so recurse on descendents (including hidden children).
            for child_layer in self.children.mut_iter() {
                match child_layer.child.add_buffers(graphics_context,
                                                    pipeline_id,
                                                    layer_id,
                                                    new_buffers,
                                                    epoch) {
                    None => return None,
                    Some(buffers) => new_buffers = buffers,
                }
            }

            // Not found. Give the caller the buffers back.
            return Some(new_buffers)
        }

        debug!("compositor_layer: layers found for add_buffers()");

        if self.epoch != epoch {
            debug!("add_buffers: compositor epoch mismatch: {:?} != {:?}, id: {:?}",
                   self.epoch,
                   epoch,
                   self.pipeline.id);
            self.pipeline.render_chan.try_send(UnusedBufferMsg(new_buffers.buffers));
            return None
        }

        {
            let quadtree = match self.quadtree {
                NoTree(..) => {
                    fail!("CompositorLayer: cannot add buffers, no quadtree initialized")
                }
                Tree(ref mut quadtree) => quadtree,
            };

            let mut unused_tiles = vec!();
            for buffer in new_buffers.buffers.move_iter().rev() {
                unused_tiles.push_all_move(quadtree.add_tile_pixel(buffer.screen_pos.origin.x,
                                                                   buffer.screen_pos.origin.y,
                                                                   buffer.resolution,
                                                                   buffer));
            }
            if !unused_tiles.is_empty() { // send back unused buffers
                self.pipeline.render_chan.try_send(UnusedBufferMsg(unused_tiles));
            }
        }

        self.build_layer_tree(graphics_context);
        return None
    }

    // Deletes a specified sublayer, including hidden children. Returns false if the layer is not
    // found.
    pub fn delete(&mut self,
                  graphics_context: &NativeCompositingGraphicsContext,
                  pipeline_id: PipelineId)
                  -> bool {
        match self.children.iter().position(|x| x.child.pipeline.id == pipeline_id) {
            Some(i) => {
                let mut child = self.children.remove(i);
                match self.quadtree {
                    NoTree(..) => {} // Nothing to do
                    Tree(ref mut quadtree) => {
                        match *child.get_ref().container.scissor.borrow() {
                            Some(rect) => {
                                quadtree.set_status_page(rect, Normal, false); // Unhide this rect
                            }
                            None => {} // Nothing to do
                        }
                    }
                }

                // Send back all tiles to renderer.
                child.get_mut_ref().child.clear_all_tiles();

                self.build_layer_tree(graphics_context);
                true
            }
            None => {
                self.children.mut_iter().map(|x| &mut x.child)
                                        .any(|x| x.delete(graphics_context, pipeline_id))
            }
        }
    }

    // Recursively sets occluded portions of quadtrees to Hidden, so that they do not ask for
    // tile requests. If layers are moved, resized, or deleted, these portions may be updated.
    fn set_occlusions(&mut self) {
        let quadtree = match self.quadtree {
            NoTree(..) => return, // Cannot calculate occlusions
            Tree(ref mut quadtree) => quadtree,
        };
        for child in self.children.iter().filter(|x| !x.child.hidden) {
            match *child.container.scissor.borrow() {
                None => {} // Nothing to do
                Some(rect) => {
                    quadtree.set_status_page(rect, Hidden, false);
                }
            }
        }
        for child in self.children.mut_iter().filter(|x| !x.child.hidden) {
            child.child.set_occlusions();
        }
    }

    /// Destroys all quadtree tiles, sending the buffers back to the renderer to be destroyed or
    /// reused.
    fn clear(&mut self) {
        match self.quadtree {
            NoTree(..) => {}
            Tree(ref mut quadtree) => {
                let mut tiles = quadtree.collect_tiles();

                // We have no way of knowing without a race whether the render task is even up and
                // running, but mark the tiles as not leaking. If the render task died, then the
                // tiles are going to be cleaned up.
                for tile in tiles.mut_iter() {
                    tile.mark_wont_leak()
                }

                self.pipeline.render_chan.try_send(UnusedBufferMsg(tiles));
            }
        }
    }

    /// Destroys tiles for this layer and all descendent layers, sending the buffers back to the
    /// renderer to be destroyed or reused.
    pub fn clear_all_tiles(&mut self) {
        self.clear();
        for kid in self.children.mut_iter() {
            kid.child.clear_all_tiles();
        }
    }

    /// Destroys all tiles of all layers, including children, *without* sending them back to the
    /// renderer. You must call this only when the render task is destined to be going down;
    /// otherwise, you will leak tiles.
    ///
    /// This is used during shutdown, when we know the render task is going away.
    pub fn forget_all_tiles(&mut self) {
        match self.quadtree {
            NoTree(..) => {}
            Tree(ref mut quadtree) => {
                let tiles = quadtree.collect_tiles();
                for tile in tiles.move_iter() {
                    let mut tile = tile;
                    tile.mark_wont_leak()
                }
            }
        }

        for kid in self.children.mut_iter() {
            kid.child.forget_all_tiles();
        }
    }

    pub fn id_of_first_child(&self) -> LayerId {
        self.children.iter().next().expect("no first child!").child.id
    }
}

