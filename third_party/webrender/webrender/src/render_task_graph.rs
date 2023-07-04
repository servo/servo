//! This module contains the render task graph.
//!
//! Code associated with creating specific render tasks is in the render_task
//! module.

use api::ImageFormat;
use api::units::*;
use crate::internal_types::{CacheTextureId, FastHashMap, SavedTargetIndex};
use crate::render_backend::FrameId;
use crate::render_target::{RenderTarget, RenderTargetKind, RenderTargetList, ColorRenderTarget};
use crate::render_target::{PictureCacheTarget, TextureCacheRenderTarget, AlphaRenderTarget};
use crate::render_task::{BlitSource, RenderTask, RenderTaskKind, RenderTaskAddress, RenderTaskData};
use crate::render_task::{RenderTaskLocation};
use crate::util::{VecHelper, Allocation};
use std::{cmp, usize, f32, i32, u32};

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RenderTaskGraph {
    pub tasks: Vec<RenderTask>,
    pub task_data: Vec<RenderTaskData>,
    /// Tasks that don't have dependencies, and that may be shared between
    /// picture tasks.
    ///
    /// We render these unconditionally before-rendering the rest of the tree.
    pub cacheable_render_tasks: Vec<RenderTaskId>,
    next_saved: SavedTargetIndex,
    frame_id: FrameId,
}

/// Allows initializing a render task directly into the render task buffer.
///
/// See utils::VecHelpers. RenderTask is fairly large so avoiding the move when
/// pushing into the vector can save a lot of exensive memcpys on pages with many
/// render tasks.
pub struct RenderTaskAllocation<'a> {
    alloc: Allocation<'a, RenderTask>,
    #[cfg(debug_assertions)]
    frame_id: FrameId,
}

impl<'l> RenderTaskAllocation<'l> {
    #[inline(always)]
    pub fn init(self, value: RenderTask) -> RenderTaskId {
        RenderTaskId {
            index: self.alloc.init(value) as u32,
            #[cfg(debug_assertions)]
            frame_id: self.frame_id,
        }
    }
}

impl RenderTaskGraph {
    pub fn new(frame_id: FrameId, counters: &RenderTaskGraphCounters) -> Self {
        // Preallocate a little more than what we needed in the previous frame so that small variations
        // in the number of items don't cause us to constantly reallocate.
        let extra_items = 8;
        RenderTaskGraph {
            tasks: Vec::with_capacity(counters.tasks_len + extra_items),
            task_data: Vec::with_capacity(counters.task_data_len + extra_items),
            cacheable_render_tasks: Vec::with_capacity(counters.cacheable_render_tasks_len + extra_items),
            next_saved: SavedTargetIndex(0),
            frame_id,
        }
    }

    pub fn counters(&self) -> RenderTaskGraphCounters {
        RenderTaskGraphCounters {
            tasks_len: self.tasks.len(),
            task_data_len: self.task_data.len(),
            cacheable_render_tasks_len: self.cacheable_render_tasks.len(),
        }
    }

    pub fn add(&mut self) -> RenderTaskAllocation {
        RenderTaskAllocation {
            alloc: self.tasks.alloc(),
            #[cfg(debug_assertions)]
            frame_id: self.frame_id,
        }
    }

    /// Express a render task dependency between a parent and child task.
    /// This is used to assign tasks to render passes.
    pub fn add_dependency(
        &mut self,
        parent_id: RenderTaskId,
        child_id: RenderTaskId,
    ) {
        let parent = &mut self[parent_id];
        parent.children.push(child_id);
    }

    /// Assign this frame's render tasks to render passes ordered so that passes appear
    /// earlier than the ones that depend on them.
    pub fn generate_passes(
        &mut self,
        main_render_task: Option<RenderTaskId>,
        screen_size: DeviceIntSize,
        gpu_supports_fast_clears: bool,
    ) -> Vec<RenderPass> {
        profile_scope!("generate_passes");
        let mut passes = Vec::new();

        if !self.cacheable_render_tasks.is_empty() {
            self.generate_passes_impl(
                &self.cacheable_render_tasks[..],
                screen_size,
                gpu_supports_fast_clears,
                false,
                &mut passes,
            );
        }

        if let Some(main_task) = main_render_task {
            self.generate_passes_impl(
                &[main_task],
                screen_size,
                gpu_supports_fast_clears,
                true,
                &mut passes,
            );
        }


        self.resolve_target_conflicts(&mut passes);

        passes
    }

    /// Assign the render tasks from the tree rooted at root_task to render passes and
    /// append them to the `passes` vector so that the passes that we depend on end up
    /// _earlier_ in the pass list.
    fn generate_passes_impl(
        &self,
        root_tasks: &[RenderTaskId],
        screen_size: DeviceIntSize,
        gpu_supports_fast_clears: bool,
        for_main_framebuffer: bool,
        passes: &mut Vec<RenderPass>,
    ) {
        // We recursively visit tasks from the roots (main and cached render tasks), to figure out
        // which ones affect the frame and which passes they should be assigned to.
        //
        // We track the maximum depth of each task (how far it is from the roots) as well as the total
        // maximum depth of the graph to determine each tasks' pass index. In a nutshell, depth 0 is
        // for the last render pass (for example the main framebuffer), while the highest depth
        // corresponds to the first pass.

        fn assign_task_depth(
            tasks: &[RenderTask],
            task_id: RenderTaskId,
            task_depth: i32,
            task_max_depths: &mut [i32],
            max_depth: &mut i32,
        ) {
            *max_depth = std::cmp::max(*max_depth, task_depth);

            let task_max_depth = &mut task_max_depths[task_id.index as usize];
            if task_depth > *task_max_depth {
                *task_max_depth = task_depth;
            } else {
                // If this task has already been processed at a larger depth,
                // there is no need to process it again.
                return;
            }

            let task = &tasks[task_id.index as usize];
            for child in &task.children {
                assign_task_depth(
                    tasks,
                    *child,
                    task_depth + 1,
                    task_max_depths,
                    max_depth,
                );
            }
        }

        // The maximum depth of each task. Values that are still equal to -1 after recursively visiting
        // the nodes correspond to tasks that don't contribute to the frame.
        let mut task_max_depths = vec![-1; self.tasks.len()];
        let mut max_depth = 0;

        for root_task in root_tasks {
            assign_task_depth(
                &self.tasks,
                *root_task,
                0,
                &mut task_max_depths,
                &mut max_depth,
            );
        }

        let offset = passes.len();

        passes.reserve(max_depth as usize + 1);
        for _ in 0..max_depth {
            passes.alloc().init(RenderPass::new_off_screen(screen_size, gpu_supports_fast_clears));
        }

        if for_main_framebuffer {
            passes.alloc().init(RenderPass::new_main_framebuffer(screen_size, gpu_supports_fast_clears));
        } else {
            passes.alloc().init(RenderPass::new_off_screen(screen_size, gpu_supports_fast_clears));
        }

        // Assign tasks to their render passes.
        for task_index in 0..self.tasks.len() {
            if task_max_depths[task_index] < 0 {
                // The task wasn't visited, it means it doesn't contribute to this frame.
                continue;
            }
            let pass_index = offset + (max_depth - task_max_depths[task_index]) as usize;
            let task_id = RenderTaskId {
                index: task_index as u32,
                #[cfg(debug_assertions)]
                frame_id: self.frame_id,
            };
            let task = &self.tasks[task_index];
            passes[pass_index as usize].add_render_task(
                task_id,
                task.get_dynamic_size(),
                task.target_kind(),
                &task.location,
            );
        }
    }

    /// Resolve conflicts between the generated passes and the limitiations of our target
    /// allocation scheme.
    ///
    /// The render task graph operates with a ping-pong target allocation scheme where
    /// a set of targets is written to by even passes and a different set of targets is
    /// written to by odd passes.
    /// Since tasks cannot read and write the same target, we can run into issues if a
    /// task pass in N + 2 reads the result of a task in pass N.
    /// To avoid such cases have to insert blit tasks to copy the content of the task
    /// into pass N + 1 which is readable by pass N + 2.
    ///
    /// In addition, allocated rects of pass N are currently not tracked and can be
    /// overwritten by allocations in later passes on the same target, unless the task
    /// has been marked for saving, which perserves the allocated rect until the end of
    /// the frame. This is a big hammer, hopefully we won't need to mark many passes
    /// for saving. A better solution would be to track allocations through the entire
    /// graph, there is a prototype of that in https://github.com/nical/toy-render-graph/
    fn resolve_target_conflicts(&mut self, passes: &mut [RenderPass]) {
        // Keep track of blit tasks we inserted to avoid adding several blits for the same
        // task.
        let mut task_redirects = vec![None; self.tasks.len()];

        let mut task_passes = vec![-1; self.tasks.len()];
        for pass_index in 0..passes.len() {
            for task in &passes[pass_index].tasks {
                task_passes[task.index as usize] = pass_index as i32;
            }
        }

        for task_index in 0..self.tasks.len() {
            if task_passes[task_index] < 0 {
                // The task doesn't contribute to this frame.
                continue;
            }

            let pass_index = task_passes[task_index];

            // Go through each dependency and check whether they belong
            // to a pass that uses the same targets and/or are more than
            // one pass behind.
            for nth_child in 0..self.tasks[task_index].children.len() {
                let child_task_index = self.tasks[task_index].children[nth_child].index as usize;
                let child_pass_index = task_passes[child_task_index];

                if child_pass_index == pass_index - 1 {
                    // This should be the most common case.
                    continue;
                }

                // TODO: Picture tasks don't support having their dependency tasks redirected.
                // Pictures store their respective render task(s) on their SurfaceInfo.
                // We cannot blit the picture task here because we would need to update the
                // surface's render tasks, but we don't have access to that info here.
                // Also a surface may be expecting a picture task and not a blit task, so
                // even if we could update the surface's render task(s), it might cause other issues.
                // For now we mark the task to be saved rather than trying to redirect to a blit task.
                let task_is_picture = if let RenderTaskKind::Picture(..) = self.tasks[task_index].kind {
                    true
                } else {
                    false
                };

                if child_pass_index % 2 != pass_index % 2 || task_is_picture {
                    // The tasks and its dependency aren't on the same targets,
                    // but the dependency needs to be kept alive.
                    self.tasks[child_task_index].mark_for_saving();
                    continue;
                }

                if let Some(blit_id) = task_redirects[child_task_index] {
                    // We already resolved a similar conflict with a blit task,
                    // reuse the same blit instead of creating a new one.
                    self.tasks[task_index].children[nth_child] = blit_id;

                    // Mark for saving if the blit is more than pass appart from
                    // our task.
                    if child_pass_index < pass_index - 2 {
                        self.tasks[blit_id.index as usize].mark_for_saving();
                    }

                    continue;
                }

                // Our dependency is an even number of passes behind, need
                // to insert a blit to ensure we don't read and write from
                // the same target.

                let child_task_id = RenderTaskId {
                    index: child_task_index as u32,
                    #[cfg(debug_assertions)]
                    frame_id: self.frame_id,
                };

                let mut blit = RenderTask::new_blit(
                    self.tasks[child_task_index].location.size(),
                    BlitSource::RenderTask { task_id: child_task_id },
                );

                // Mark for saving if the blit is more than pass appart from
                // our task.
                if child_pass_index < pass_index - 2 {
                    blit.mark_for_saving();
                }

                let blit_id = RenderTaskId {
                    index: self.tasks.len() as u32,
                    #[cfg(debug_assertions)]
                    frame_id: self.frame_id,
                };

                self.tasks.alloc().init(blit);

                passes[child_pass_index as usize + 1].tasks.push(blit_id);

                self.tasks[task_index].children[nth_child] = blit_id;
                task_redirects[child_task_index] = Some(blit_id);
            }
        }
    }

    pub fn get_task_address(&self, id: RenderTaskId) -> RenderTaskAddress {
        #[cfg(all(debug_assertions, not(feature = "replay")))]
        debug_assert_eq!(self.frame_id, id.frame_id);
        RenderTaskAddress(id.index as u16)
    }

    pub fn write_task_data(&mut self) {
        profile_scope!("write_task_data");
        for task in &self.tasks {
            self.task_data.push(task.write_task_data());
        }
    }

    pub fn save_target(&mut self) -> SavedTargetIndex {
        let id = self.next_saved;
        self.next_saved.0 += 1;
        id
    }

    #[cfg(debug_assertions)]
    pub fn frame_id(&self) -> FrameId {
        self.frame_id
    }
}

impl std::ops::Index<RenderTaskId> for RenderTaskGraph {
    type Output = RenderTask;
    fn index(&self, id: RenderTaskId) -> &RenderTask {
        #[cfg(all(debug_assertions, not(feature = "replay")))]
        debug_assert_eq!(self.frame_id, id.frame_id);
        &self.tasks[id.index as usize]
    }
}

impl std::ops::IndexMut<RenderTaskId> for RenderTaskGraph {
    fn index_mut(&mut self, id: RenderTaskId) -> &mut RenderTask {
        #[cfg(all(debug_assertions, not(feature = "replay")))]
        debug_assert_eq!(self.frame_id, id.frame_id);
        &mut self.tasks[id.index as usize]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RenderTaskId {
    pub index: u32,

    #[cfg(debug_assertions)]
    #[cfg_attr(feature = "replay", serde(default = "FrameId::first"))]
    frame_id: FrameId,
}

#[derive(Debug)]
pub struct RenderTaskGraphCounters {
    tasks_len: usize,
    task_data_len: usize,
    cacheable_render_tasks_len: usize,
}

impl RenderTaskGraphCounters {
    pub fn new() -> Self {
        RenderTaskGraphCounters {
            tasks_len: 0,
            task_data_len: 0,
            cacheable_render_tasks_len: 0,
        }
    }
}

impl RenderTaskId {
    pub const INVALID: RenderTaskId = RenderTaskId {
        index: u32::MAX,
        #[cfg(debug_assertions)]
        frame_id: FrameId::INVALID,
    };
}

/// Contains the set of `RenderTarget`s specific to the kind of pass.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum RenderPassKind {
    /// The final pass to the main frame buffer, where we have a single color
    /// target for display to the user.
    MainFramebuffer {
        main_target: ColorRenderTarget,
    },
    /// An intermediate pass, where we may have multiple targets.
    OffScreen {
        alpha: RenderTargetList<AlphaRenderTarget>,
        color: RenderTargetList<ColorRenderTarget>,
        texture_cache: FastHashMap<(CacheTextureId, usize), TextureCacheRenderTarget>,
        picture_cache: Vec<PictureCacheTarget>,
    },
}

/// A render pass represents a set of rendering operations that don't depend on one
/// another.
///
/// A render pass can have several render targets if there wasn't enough space in one
/// target to do all of the rendering for that pass. See `RenderTargetList`.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RenderPass {
    /// The kind of pass, as well as the set of targets associated with that
    /// kind of pass.
    pub kind: RenderPassKind,
    /// The set of tasks to be performed in this pass, as indices into the
    /// `RenderTaskGraph`.
    pub tasks: Vec<RenderTaskId>,
    /// Screen size in device pixels - used for opaque alpha batch break threshold.
    pub screen_size: DeviceIntSize,
}

impl RenderPass {
    /// Creates a pass for the main framebuffer. There is only one of these, and
    /// it is always the last pass.
    pub fn new_main_framebuffer(
        screen_size: DeviceIntSize,
        gpu_supports_fast_clears: bool,
    ) -> Self {
        let main_target = ColorRenderTarget::new(screen_size, gpu_supports_fast_clears);
        RenderPass {
            kind: RenderPassKind::MainFramebuffer {
                main_target,
            },
            tasks: vec![],
            screen_size,
        }
    }

    /// Creates an intermediate off-screen pass.
    pub fn new_off_screen(
        screen_size: DeviceIntSize,
        gpu_supports_fast_clears: bool,
    ) -> Self {
        RenderPass {
            kind: RenderPassKind::OffScreen {
                color: RenderTargetList::new(
                    screen_size,
                    ImageFormat::RGBA8,
                    gpu_supports_fast_clears,
                ),
                alpha: RenderTargetList::new(
                    screen_size,
                    ImageFormat::R8,
                    gpu_supports_fast_clears,
                ),
                texture_cache: FastHashMap::default(),
                picture_cache: Vec::new(),
            },
            tasks: vec![],
            screen_size,
        }
    }

    /// Adds a task to this pass.
    pub fn add_render_task(
        &mut self,
        task_id: RenderTaskId,
        size: DeviceIntSize,
        target_kind: RenderTargetKind,
        location: &RenderTaskLocation,
    ) {
        if let RenderPassKind::OffScreen { ref mut color, ref mut alpha, .. } = self.kind {
            // If this will be rendered to a dynamically-allocated region on an
            // off-screen render target, update the max-encountered size. We don't
            // need to do this for things drawn to the texture cache, since those
            // don't affect our render target allocation.
            if location.is_dynamic() {
                let max_size = match target_kind {
                    RenderTargetKind::Color => &mut color.max_dynamic_size,
                    RenderTargetKind::Alpha => &mut alpha.max_dynamic_size,
                };
                max_size.width = cmp::max(max_size.width, size.width);
                max_size.height = cmp::max(max_size.height, size.height);
            }
        }

        self.tasks.push(task_id);
    }
}

// Dump an SVG visualization of the render graph for debugging purposes
#[allow(dead_code)]
pub fn dump_render_tasks_as_svg(
    render_tasks: &RenderTaskGraph,
    passes: &[RenderPass],
    output: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    use svg_fmt::*;

    let node_width = 80.0;
    let node_height = 30.0;
    let vertical_spacing = 8.0;
    let horizontal_spacing = 20.0;
    let margin = 10.0;
    let text_size = 10.0;

    let mut pass_rects = Vec::new();
    let mut nodes = vec![None; render_tasks.tasks.len()];

    let mut x = margin;
    let mut max_y: f32 = 0.0;

    #[derive(Clone)]
    struct Node {
        rect: Rectangle,
        label: Text,
        size: Text,
    }

    for pass in passes {
        let mut layout = VerticalLayout::new(x, margin, node_width);

        for task_id in &pass.tasks {
            let task_index = task_id.index as usize;
            let task = &render_tasks.tasks[task_index];

            let rect = layout.push_rectangle(node_height);

            let tx = rect.x + rect.w / 2.0;
            let ty = rect.y + 10.0;

            let saved = if task.saved_index.is_some() { " (Saved)" } else { "" };
            let label = text(tx, ty, format!("{}{}", task.kind.as_str(), saved));
            let size = text(tx, ty + 12.0, format!("{:?}", task.location.size()));

            nodes[task_index] = Some(Node { rect, label, size });

            layout.advance(vertical_spacing);
        }

        pass_rects.push(layout.total_rectangle());

        x += node_width + horizontal_spacing;
        max_y = max_y.max(layout.y + margin);
    }

    let mut links = Vec::new();
    for node_index in 0..nodes.len() {
        if nodes[node_index].is_none() {
            continue;
        }

        let task = &render_tasks.tasks[node_index];
        for dep in &task.children {
            let dep_index = dep.index as usize;

            if let (&Some(ref node), &Some(ref dep_node)) = (&nodes[node_index], &nodes[dep_index]) {
                links.push((
                    dep_node.rect.x + dep_node.rect.w,
                    dep_node.rect.y + dep_node.rect.h / 2.0,
                    node.rect.x,
                    node.rect.y + node.rect.h / 2.0,
                ));
            }
        }
    }

    let svg_w = x + margin;
    let svg_h = max_y + margin;
    writeln!(output, "{}", BeginSvg { w: svg_w, h: svg_h })?;

    // Background.
    writeln!(output,
        "    {}",
        rectangle(0.0, 0.0, svg_w, svg_h)
            .inflate(1.0, 1.0)
            .fill(rgb(50, 50, 50))
    )?;

    // Passes.
    for rect in pass_rects {
        writeln!(output,
            "    {}",
            rect.inflate(3.0, 3.0)
                .border_radius(4.0)
                .opacity(0.4)
                .fill(black())
        )?;
    }

    // Links.
    for (x1, y1, x2, y2) in links {
        dump_task_dependency_link(output, x1, y1, x2, y2);
    }

    // Tasks.
    for node in &nodes {
        if let Some(node) = node {
            writeln!(output,
                "    {}",
                node.rect
                    .clone()
                    .fill(black())
                    .border_radius(3.0)
                    .opacity(0.5)
                    .offset(0.0, 2.0)
            )?;
            writeln!(output,
                "    {}",
                node.rect
                    .clone()
                    .fill(rgb(200, 200, 200))
                    .border_radius(3.0)
                    .opacity(0.8)
            )?;

            writeln!(output,
                "    {}",
                node.label
                    .clone()
                    .size(text_size)
                    .align(Align::Center)
                    .color(rgb(50, 50, 50))
            )?;
            writeln!(output,
                "    {}",
                node.size
                    .clone()
                    .size(text_size * 0.7)
                    .align(Align::Center)
                    .color(rgb(50, 50, 50))
            )?;
        }
    }

    writeln!(output, "{}", EndSvg)
}

#[allow(dead_code)]
fn dump_task_dependency_link(
    output: &mut dyn std::io::Write,
    x1: f32, y1: f32,
    x2: f32, y2: f32,
) {
    use svg_fmt::*;

    // If the link is a straight horizontal line and spans over multiple passes, it
    // is likely to go straight though unrelated nodes in a way that makes it look like
    // they are connected, so we bend the line upward a bit to avoid that.
    let simple_path = (y1 - y2).abs() > 1.0 || (x2 - x1) < 45.0;

    let mid_x = (x1 + x2) / 2.0;
    if simple_path {
        write!(output, "    {}",
            path().move_to(x1, y1)
                .cubic_bezier_to(mid_x, y1, mid_x, y2, x2, y2)
                .fill(Fill::None)
                .stroke(Stroke::Color(rgb(100, 100, 100), 3.0))
        ).unwrap();
    } else {
        let ctrl1_x = (mid_x + x1) / 2.0;
        let ctrl2_x = (mid_x + x2) / 2.0;
        let ctrl_y = y1 - 25.0;
        write!(output, "    {}",
            path().move_to(x1, y1)
                .cubic_bezier_to(ctrl1_x, y1, ctrl1_x, ctrl_y, mid_x, ctrl_y)
                .cubic_bezier_to(ctrl2_x, ctrl_y, ctrl2_x, y2, x2, y2)
                .fill(Fill::None)
                .stroke(Stroke::Color(rgb(100, 100, 100), 3.0))
        ).unwrap();
    }
}

#[cfg(test)]
use euclid::{size2, rect};
#[cfg(test)]
use smallvec::SmallVec;

#[cfg(test)]
fn dyn_location(w: i32, h: i32) -> RenderTaskLocation {
    RenderTaskLocation::Dynamic(None, size2(w, h))
}

#[test]
fn diamond_task_graph() {
    // A simple diamon shaped task graph.
    //
    //     [b1]
    //    /    \
    // [a]      [main_pic]
    //    \    /
    //     [b2]

    let color = RenderTargetKind::Color;

    let counters = RenderTaskGraphCounters::new();
    let mut tasks = RenderTaskGraph::new(FrameId::first(), &counters);

    let a = tasks.add().init(RenderTask::new_test(color, dyn_location(640, 640), SmallVec::new()));
    let b1 = tasks.add().init(RenderTask::new_test(color, dyn_location(320, 320), smallvec![a]));
    let b2 = tasks.add().init(RenderTask::new_test(color, dyn_location(320, 320), smallvec![a]));

    let main_pic = tasks.add().init(RenderTask::new_test(
        color,
        RenderTaskLocation::Fixed(rect(0, 0, 3200, 1800)),
        smallvec![b1, b2],
    ));

    let initial_number_of_tasks = tasks.tasks.len();

    let passes = tasks.generate_passes(Some(main_pic), size2(3200, 1800), true);

    // We should not have added any blits.
    assert_eq!(tasks.tasks.len(), initial_number_of_tasks);

    assert_eq!(passes.len(), 3);
    assert_eq!(passes[0].tasks, vec![a]);

    assert_eq!(passes[1].tasks.len(), 2);
    assert!(passes[1].tasks.contains(&b1));
    assert!(passes[1].tasks.contains(&b2));

    assert_eq!(passes[2].tasks, vec![main_pic]);
}

#[test]
fn blur_task_graph() {
    // This test simulates a complicated shadow stack effect with target allocation
    // conflicts to resolve.

    let color = RenderTargetKind::Color;

    let counters = RenderTaskGraphCounters::new();
    let mut tasks = RenderTaskGraph::new(FrameId::first(), &counters);

    let pic = tasks.add().init(RenderTask::new_test(color, dyn_location(640, 640), SmallVec::new()));
    let scale1 = tasks.add().init(RenderTask::new_test(color, dyn_location(320, 320), smallvec![pic]));
    let scale2 = tasks.add().init(RenderTask::new_test(color, dyn_location(160, 160), smallvec![scale1]));
    let scale3 = tasks.add().init(RenderTask::new_test(color, dyn_location(80, 80), smallvec![scale2]));
    let scale4 = tasks.add().init(RenderTask::new_test(color, dyn_location(40, 40), smallvec![scale3]));

    let vblur1 = tasks.add().init(RenderTask::new_test(color, dyn_location(40, 40), smallvec![scale4]));
    let hblur1 = tasks.add().init(RenderTask::new_test(color, dyn_location(40, 40), smallvec![vblur1]));

    let vblur2 = tasks.add().init(RenderTask::new_test(color, dyn_location(40, 40), smallvec![scale4]));
    let hblur2 = tasks.add().init(RenderTask::new_test(color, dyn_location(40, 40), smallvec![vblur2]));

    // Insert a task that is an even number of passes away from its dependency.
    // This means the source and destination are on the same target and we have to resolve
    // this conflict by automatically inserting a blit task.
    let vblur3 = tasks.add().init(RenderTask::new_test(color, dyn_location(80, 80), smallvec![scale3]));
    let hblur3 = tasks.add().init(RenderTask::new_test(color, dyn_location(80, 80), smallvec![vblur3]));

    // Insert a task that is an odd number > 1 of passes away from its dependency.
    // This should force us to mark the dependency "for saving" to keep its content valid
    // until the task can access it.
    let vblur4 = tasks.add().init(RenderTask::new_test(color, dyn_location(160, 160), smallvec![scale2]));
    let hblur4 = tasks.add().init(RenderTask::new_test(color, dyn_location(160, 160), smallvec![vblur4]));

    let main_pic = tasks.add().init(RenderTask::new_test(
        color,
        RenderTaskLocation::Fixed(rect(0, 0, 3200, 1800)),
        smallvec![hblur1, hblur2, hblur3, hblur4],
    ));

    let initial_number_of_tasks = tasks.tasks.len();

    let passes = tasks.generate_passes(Some(main_pic), size2(3200, 1800), true);

    // We should have added a single blit task.
    assert_eq!(tasks.tasks.len(), initial_number_of_tasks + 1);

    // vblur3's dependency to scale3 should be replaced by a blit.
    let blit = tasks[vblur3].children[0];
    assert!(blit != scale3);

    match tasks[blit].kind {
        RenderTaskKind::Blit(..) => {}
        _ => { panic!("This should be a blit task."); }
    }

    assert_eq!(passes.len(), 8);

    assert_eq!(passes[0].tasks, vec![pic]);
    assert_eq!(passes[1].tasks, vec![scale1]);
    assert_eq!(passes[2].tasks, vec![scale2]);
    assert_eq!(passes[3].tasks, vec![scale3]);

    assert_eq!(passes[4].tasks.len(), 2);
    assert!(passes[4].tasks.contains(&scale4));
    assert!(passes[4].tasks.contains(&blit));

    assert_eq!(passes[5].tasks.len(), 4);
    assert!(passes[5].tasks.contains(&vblur1));
    assert!(passes[5].tasks.contains(&vblur2));
    assert!(passes[5].tasks.contains(&vblur3));
    assert!(passes[5].tasks.contains(&vblur4));

    assert_eq!(passes[6].tasks.len(), 4);
    assert!(passes[6].tasks.contains(&hblur1));
    assert!(passes[6].tasks.contains(&hblur2));
    assert!(passes[6].tasks.contains(&hblur3));
    assert!(passes[6].tasks.contains(&hblur4));

    assert_eq!(passes[7].tasks, vec![main_pic]);

    // See vblur4's comment above.
    assert!(tasks[scale2].saved_index.is_some());
}

#[test]
fn culled_tasks() {
    // This test checks that tasks that do not contribute to the frame don't appear in the
    // generated passes.

    let color = RenderTargetKind::Color;

    let counters = RenderTaskGraphCounters::new();
    let mut tasks = RenderTaskGraph::new(FrameId::first(), &counters);

    let a1 = tasks.add().init(RenderTask::new_test(color, dyn_location(640, 640), SmallVec::new()));
    let _a2 = tasks.add().init(RenderTask::new_test(color, dyn_location(320, 320), smallvec![a1]));

    let b1 = tasks.add().init(RenderTask::new_test(color, dyn_location(640, 640), SmallVec::new()));
    let b2 = tasks.add().init(RenderTask::new_test(color, dyn_location(320, 320), smallvec![b1]));
    let _b3 = tasks.add().init(RenderTask::new_test(color, dyn_location(320, 320), smallvec![b2]));

    let main_pic = tasks.add().init(RenderTask::new_test(
        color,
        RenderTaskLocation::Fixed(rect(0, 0, 3200, 1800)),
        smallvec![b2],
    ));

    let initial_number_of_tasks = tasks.tasks.len();

    let passes = tasks.generate_passes(Some(main_pic), size2(3200, 1800), true);

    // We should not have added any blits.
    assert_eq!(tasks.tasks.len(), initial_number_of_tasks);

    assert_eq!(passes.len(), 3);
    assert_eq!(passes[0].tasks, vec![b1]);
    assert_eq!(passes[1].tasks, vec![b2]);
    assert_eq!(passes[2].tasks, vec![main_pic]);
}
