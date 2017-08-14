/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate gfx;
extern crate gfx_traits;
extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate msg;
extern crate profile_traits;
extern crate script_traits;
extern crate servo_config;

use gfx::display_list::{DisplayItem, DisplayList};
use gfx_traits::Epoch;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use profile_traits::time::{ProfilerChan, ProfilerCategory, send_profile_data};
use profile_traits::time::TimerMetadata;
use script_traits::{ConstellationControlMsg, LayoutMsg, PaintMetricType};
use servo_config::opts;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

pub trait ProfilerMetadataFactory {
    fn new_metadata(&self) -> Option<TimerMetadata>;
}

macro_rules! make_time_setter(
    ( $attr:ident, $func:ident, $category:ident, $label:expr, $metric_type:path ) => (
        fn $func(&self,
                 profiler_metadata: Option<TimerMetadata>,
                 paint_time: f64) {
            if self.$attr.get().is_some() {
                return;
            }

            let navigation_start = match self.navigation_start {
                Some(time) => time,
                None => {
                    warn!("Trying to set metric before navigation start");
                    return;
                }
            };

            let time = paint_time - navigation_start;
            self.$attr.set(Some(time));

            // Queue performance observer notification.
            let msg = ConstellationControlMsg::PaintMetric(self.pipeline_id,
                                                           $metric_type,
                                                           time);
            if let Err(e) = self.script_chan.send(msg) {
                warn!("Sending paint metric to script thread failed ({}).", e);
            }

            // Send the metric to the time profiler.
            send_profile_data(ProfilerCategory::$category,
                              profiler_metadata,
                              &self.time_profiler_chan,
                              time as u64, time as u64, 0, 0);

            // Print the metric to console if the print-pwm option was given.
            if opts::get().print_pwm {
                println!("{:?} {:?}", $label, time);
            }
        }
    );
);

pub struct PaintTimeMetrics {
    pending_metrics: RefCell<HashMap<Epoch, (Option<TimerMetadata>, bool)>>,
    navigation_start: Option<f64>,
    first_paint: Cell<Option<f64>>,
    first_contentful_paint: Cell<Option<f64>>,
    pipeline_id: PipelineId,
    time_profiler_chan: ProfilerChan,
    constellation_chan: IpcSender<LayoutMsg>,
    script_chan: IpcSender<ConstellationControlMsg>,
}

impl PaintTimeMetrics {
    pub fn new(pipeline_id: PipelineId,
               time_profiler_chan: ProfilerChan,
               constellation_chan: IpcSender<LayoutMsg>,
               script_chan: IpcSender<ConstellationControlMsg>)
        -> PaintTimeMetrics {
        PaintTimeMetrics {
            pending_metrics: RefCell::new(HashMap::new()),
            navigation_start: None,
            first_paint: Cell::new(None),
            first_contentful_paint: Cell::new(None),
            pipeline_id,
            time_profiler_chan,
            constellation_chan,
            script_chan,
        }
    }

    pub fn set_navigation_start(&mut self, time: f64) {
        self.navigation_start = Some(time);
    }

    make_time_setter!(first_paint, set_first_paint,
                      TimeToFirstPaint,
                      "first-paint",
                      PaintMetricType::FirstPaint);
    make_time_setter!(first_contentful_paint, set_first_contentful_paint,
                      TimeToFirstContentfulPaint,
                      "first-contentful-paint",
                      PaintMetricType::FirstContentfulPaint);

    pub fn maybe_observe_paint_time<T>(&self,
                                       profiler_metadata_factory: &T,
                                       epoch: Epoch,
                                       display_list: &DisplayList)
        where T: ProfilerMetadataFactory {
        if self.first_paint.get().is_some() && self.first_contentful_paint.get().is_some() {
            // If we already set all paint metrics, we just bail out.
            return;
        }

        let mut is_contentful = false;
        // Analyze the display list to figure out if this may be the first
        // contentful paint (i.e. the display list contains items of type text,
        // image, non-white canvas or SVG).
        for item in &display_list.list {
            match item {
                &DisplayItem::Text(_) |
                &DisplayItem::Image(_) => {
                    is_contentful = true;
                    break;
                },
                _ => (),
            }
        }

        self.pending_metrics.borrow_mut().insert(
            epoch,
            (profiler_metadata_factory.new_metadata(), is_contentful)
        );

        // Send the pending metric information to the compositor thread.
        // The compositor will record the current time after painting the
        // frame with the given ID and will send the metric back to us.
        let msg = LayoutMsg::PendingPaintMetric(self.pipeline_id, epoch);
        if let Err(e) = self.constellation_chan.send(msg) {
            warn!("Failed to send PendingPaintMetric {:?}", e);
        }
    }

    pub fn maybe_set_metric(&mut self, epoch: Epoch, paint_time: f64) {
        if (self.first_paint.get().is_some() && self.first_contentful_paint.get().is_some()) ||
           self.navigation_start.is_none() {
            // If we already set all paint metrics or we have not set navigation start yet,
            // we just bail out.
            return;
        }

        if let Some(pending_metric) = self.pending_metrics.borrow_mut().remove(&epoch) {
            let profiler_metadata = pending_metric.0;
            self.set_first_paint(profiler_metadata.clone(), paint_time);
            if pending_metric.1 {
                self.set_first_contentful_paint(profiler_metadata, paint_time);
            }
        }

    }

    pub fn get_navigation_start(&self) -> Option<f64> {
        self.navigation_start
    }

    pub fn get_first_paint(&self) -> Option<f64> {
        self.first_paint.get()
    }

    pub fn get_first_contentful_paint(&self) -> Option<f64> {
        self.first_contentful_paint.get()
    }
}
