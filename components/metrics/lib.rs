/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate gfx;
extern crate profile_traits;
extern crate servo_config;
extern crate time;

use gfx::display_list::{DisplayItem, DisplayList};
use profile_traits::time::{ProfilerChan, ProfilerCategory, send_profile_data};
use profile_traits::time::TimerMetadata;
use servo_config::opts;
use std::cell::Cell;

pub trait ProfilerMetadataFactory {
    fn new_metadata(&self) -> Option<TimerMetadata>;
}

macro_rules! make_time_setter(
    ( $attr:ident, $func:ident, $category:ident, $label:expr ) => (
        fn $func<T>(&self, profiler_metadata_factory: &T)
            where T: ProfilerMetadataFactory {
            let navigation_start = match self.navigation_start {
                Some(time) => time,
                None => {
                    println!("Trying to set metric before navigation start");
                    return;
                }
            };

            let now = time::precise_time_ns() as f64;
            let time = now - navigation_start;
            self.$attr.set(Some(time));

            // Send the metric to the time profiler.
            send_profile_data(ProfilerCategory::$category,
                              profiler_metadata_factory.new_metadata(),
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
    navigation_start: Option<f64>,
    first_paint: Cell<Option<f64>>,
    first_contentful_paint: Cell<Option<f64>>,
    time_profiler_chan: ProfilerChan,
}

impl PaintTimeMetrics {
    pub fn new(time_profiler_chan: ProfilerChan)
        -> PaintTimeMetrics {
        PaintTimeMetrics {
            navigation_start: None,
            first_paint: Cell::new(None),
            first_contentful_paint: Cell::new(None),
            time_profiler_chan: time_profiler_chan,
        }
    }

    pub fn set_navigation_start(&mut self, time: f64) {
        self.navigation_start = Some(time);
    }

    make_time_setter!(first_paint, set_first_paint,
                      TimeToFirstPaint,
                      "first-paint");
    make_time_setter!(first_contentful_paint, set_first_contentful_paint,
                      TimeToFirstContentfulPaint,
                      "first-contentful-paint");

    pub fn maybe_set_first_paint<T>(&self, profiler_metadata_factory: &T)
        where T: ProfilerMetadataFactory {
        {
            if self.first_paint.get().is_some() {
                return;
            }
        }

        self.set_first_paint(profiler_metadata_factory);
    }

    pub fn maybe_set_first_contentful_paint<T>(&self, profiler_metadata_factory: &T,
                                               display_list: &DisplayList)
        where T: ProfilerMetadataFactory {
        {
            if self.first_contentful_paint.get().is_some() {
                return;
            }
        }

        // Analyze display list to figure out if this is the first contentful
        // paint (i.e. the display list contains items of type text, image,
        // non-white canvas or SVG)
        for item in &display_list.list {
            match item {
                &DisplayItem::Text(_) |
                &DisplayItem::Image(_) => {
                    self.set_first_contentful_paint(profiler_metadata_factory);
                    return;
                },
                _ => (),
            }
        }
    }
}
