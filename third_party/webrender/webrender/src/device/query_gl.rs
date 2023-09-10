/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gleam::gl;
use std::mem;
use std::rc::Rc;

use crate::device::GpuFrameId;
use crate::profiler::GpuProfileTag;

#[derive(Copy, Clone, Debug)]
pub enum GpuDebugMethod {
    None,
    MarkerEXT,
    KHR,
}

#[derive(Debug, Clone)]
pub struct GpuTimer {
    pub tag: GpuProfileTag,
    pub time_ns: u64,
}

#[derive(Debug, Clone)]
pub struct GpuSampler {
    pub tag: GpuProfileTag,
    pub count: u64,
}

pub struct QuerySet<T> {
    set: Vec<gl::GLuint>,
    data: Vec<T>,
    pending: gl::GLuint,
}

impl<T> QuerySet<T> {
    fn new() -> Self {
        QuerySet {
            set: Vec::new(),
            data: Vec::new(),
            pending: 0,
        }
    }

    fn reset(&mut self) {
        self.data.clear();
        self.pending = 0;
    }

    fn add(&mut self, value: T) -> Option<gl::GLuint> {
        assert_eq!(self.pending, 0);
        self.set.get(self.data.len()).cloned().map(|query_id| {
            self.data.push(value);
            self.pending = query_id;
            query_id
        })
    }

    fn take<F: Fn(&mut T, gl::GLuint)>(&mut self, fun: F) -> Vec<T> {
        let mut data = mem::replace(&mut self.data, Vec::new());
        for (value, &query) in data.iter_mut().zip(self.set.iter()) {
            fun(value, query)
        }
        data
    }
}

pub struct GpuFrameProfile {
    gl: Rc<dyn gl::Gl>,
    timers: QuerySet<GpuTimer>,
    samplers: QuerySet<GpuSampler>,
    frame_id: GpuFrameId,
    inside_frame: bool,
    debug_method: GpuDebugMethod,
}

impl GpuFrameProfile {
    fn new(gl: Rc<dyn gl::Gl>, debug_method: GpuDebugMethod) -> Self {
        GpuFrameProfile {
            gl,
            timers: QuerySet::new(),
            samplers: QuerySet::new(),
            frame_id: GpuFrameId::new(0),
            inside_frame: false,
            debug_method
        }
    }

    fn enable_timers(&mut self, count: i32) {
        self.timers.set = self.gl.gen_queries(count);
    }

    fn disable_timers(&mut self) {
        if !self.timers.set.is_empty() {
            self.gl.delete_queries(&self.timers.set);
        }
        self.timers.set = Vec::new();
    }

    fn enable_samplers(&mut self, count: i32) {
        self.samplers.set = self.gl.gen_queries(count);
    }

    fn disable_samplers(&mut self) {
        if !self.samplers.set.is_empty() {
            self.gl.delete_queries(&self.samplers.set);
        }
        self.samplers.set = Vec::new();
    }

    fn begin_frame(&mut self, frame_id: GpuFrameId) {
        self.frame_id = frame_id;
        self.timers.reset();
        self.samplers.reset();
        self.inside_frame = true;
    }

    fn end_frame(&mut self) {
        self.finish_timer();
        self.finish_sampler();
        self.inside_frame = false;
    }

    fn finish_timer(&mut self) {
        debug_assert!(self.inside_frame);
        if self.timers.pending != 0 {
            self.gl.end_query(gl::TIME_ELAPSED);
            self.timers.pending = 0;
        }
    }

    fn finish_sampler(&mut self) {
        debug_assert!(self.inside_frame);
        if self.samplers.pending != 0 {
            self.gl.end_query(gl::SAMPLES_PASSED);
            self.samplers.pending = 0;
        }
    }

    fn start_timer(&mut self, tag: GpuProfileTag) -> GpuTimeQuery {
        self.finish_timer();

        let marker = GpuMarker::new(&self.gl, tag.label, self.debug_method);

        if let Some(query) = self.timers.add(GpuTimer { tag, time_ns: 0 }) {
            self.gl.begin_query(gl::TIME_ELAPSED, query);
        }

        GpuTimeQuery(marker)
    }

    fn start_sampler(&mut self, tag: GpuProfileTag) -> GpuSampleQuery {
        self.finish_sampler();

        if let Some(query) = self.samplers.add(GpuSampler { tag, count: 0 }) {
            self.gl.begin_query(gl::SAMPLES_PASSED, query);
        }

        GpuSampleQuery
    }

    fn build_samples(&mut self) -> (GpuFrameId, Vec<GpuTimer>, Vec<GpuSampler>) {
        debug_assert!(!self.inside_frame);
        let gl = &self.gl;

        (
            self.frame_id,
            self.timers.take(|timer, query| {
                timer.time_ns = gl.get_query_object_ui64v(query, gl::QUERY_RESULT)
            }),
            self.samplers.take(|sampler, query| {
                sampler.count = gl.get_query_object_ui64v(query, gl::QUERY_RESULT)
            }),
        )
    }
}

impl Drop for GpuFrameProfile {
    fn drop(&mut self) {
        self.disable_timers();
        self.disable_samplers();
    }
}

const NUM_PROFILE_FRAMES: usize = 4;

pub struct GpuProfiler {
    gl: Rc<dyn gl::Gl>,
    frames: [GpuFrameProfile; NUM_PROFILE_FRAMES],
    next_frame: usize,
    debug_method: GpuDebugMethod
}

impl GpuProfiler {
    pub fn new(gl: Rc<dyn gl::Gl>, debug_method: GpuDebugMethod) -> Self {
        let f = || GpuFrameProfile::new(Rc::clone(&gl), debug_method);

        let frames = [f(), f(), f(), f()];
        GpuProfiler {
            gl,
            next_frame: 0,
            frames,
            debug_method
        }
    }

    pub fn enable_timers(&mut self) {
        const MAX_TIMERS_PER_FRAME: i32 = 256;

        for frame in &mut self.frames {
            frame.enable_timers(MAX_TIMERS_PER_FRAME);
        }
    }

    pub fn disable_timers(&mut self) {
        for frame in &mut self.frames {
            frame.disable_timers();
        }
    }

    pub fn enable_samplers(&mut self) {
        const MAX_SAMPLERS_PER_FRAME: i32 = 16;
        if cfg!(target_os = "macos") {
            warn!("Expect macOS driver bugs related to sample queries")
        }

        for frame in &mut self.frames {
            frame.enable_samplers(MAX_SAMPLERS_PER_FRAME);
        }
    }

    pub fn disable_samplers(&mut self) {
        for frame in &mut self.frames {
            frame.disable_samplers();
        }
    }

    pub fn build_samples(&mut self) -> (GpuFrameId, Vec<GpuTimer>, Vec<GpuSampler>) {
        self.frames[self.next_frame].build_samples()
    }

    pub fn begin_frame(&mut self, frame_id: GpuFrameId) {
        self.frames[self.next_frame].begin_frame(frame_id);
    }

    pub fn end_frame(&mut self) {
        self.frames[self.next_frame].end_frame();
        self.next_frame = (self.next_frame + 1) % self.frames.len();
    }

    pub fn start_timer(&mut self, tag: GpuProfileTag) -> GpuTimeQuery {
        self.frames[self.next_frame].start_timer(tag)
    }

    pub fn start_sampler(&mut self, tag: GpuProfileTag) -> GpuSampleQuery {
        self.frames[self.next_frame].start_sampler(tag)
    }

    pub fn finish_sampler(&mut self, _sampler: GpuSampleQuery) {
        self.frames[self.next_frame].finish_sampler()
    }

    pub fn start_marker(&mut self, label: &str) -> GpuMarker {
        GpuMarker::new(&self.gl, label, self.debug_method)
    }

    pub fn place_marker(&mut self, label: &str) {
        GpuMarker::fire(&self.gl, label, self.debug_method)
    }
}

#[must_use]
pub struct GpuMarker {
    gl: Option<(Rc<dyn gl::Gl>, GpuDebugMethod)>,
}

impl GpuMarker {
    fn new(gl: &Rc<dyn gl::Gl>, message: &str, debug_method: GpuDebugMethod) -> Self {
        let gl = match debug_method {
            GpuDebugMethod::KHR => {
              gl.push_debug_group_khr(gl::DEBUG_SOURCE_APPLICATION, 0, message);
              Some((Rc::clone(gl), debug_method))
            },
            GpuDebugMethod::MarkerEXT => {
              gl.push_group_marker_ext(message);
              Some((Rc::clone(gl), debug_method))
            },
            GpuDebugMethod::None => None,
        };
        GpuMarker { gl }
    }

    fn fire(gl: &Rc<dyn gl::Gl>, message: &str, debug_method: GpuDebugMethod) {
        match debug_method {
            GpuDebugMethod::KHR => gl.debug_message_insert_khr(gl::DEBUG_SOURCE_APPLICATION, gl::DEBUG_TYPE_MARKER, 0, gl::DEBUG_SEVERITY_NOTIFICATION, message),
            GpuDebugMethod::MarkerEXT => gl.insert_event_marker_ext(message),
            GpuDebugMethod::None => {}
        };
    }
}

impl Drop for GpuMarker {
    fn drop(&mut self) {
        if let Some((ref gl, debug_method)) = self.gl {
            match debug_method {
                GpuDebugMethod::KHR => gl.pop_debug_group_khr(),
                GpuDebugMethod::MarkerEXT => gl.pop_group_marker_ext(),
                GpuDebugMethod::None => {}
            };
        }
    }
}

#[must_use]
pub struct GpuTimeQuery(GpuMarker);
#[must_use]
pub struct GpuSampleQuery;
