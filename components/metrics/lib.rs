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
use std::sync::{Arc, Mutex, RwLock};

pub trait ProfilerMetadataFactory {
    fn new_metadata(&self) -> Option<TimerMetadata>;
}

macro_rules! make_time_setter(
    ( $attr:ident, $func:ident, $category:ident, $label:expr ) => (
        fn $func<T>(&self, profiler_metadata_factory: T)
            where T: ProfilerMetadataFactory {
            let navigation_start = match *self.navigation_start.read().unwrap() {
                Some(time) => time,
                None => {
                    println!("Trying to set metric before navigation start");
                    return;
                }
            };

            let now = time::precise_time_ns() as f64;
            let time = now - navigation_start;
            {
                let mut attr = self.$attr.write().unwrap();
                *attr = Some(time);
            }

            // Send the metric to the time profiler.
            {
                let profiler_chan = self.time_profiler_chan.lock().unwrap();
                send_profile_data(ProfilerCategory::$category,
                                  profiler_metadata_factory.new_metadata(),
                                  &profiler_chan,
                                  time as u64, time as u64, 0, 0);
            }

            // Print the metric to console if the print-pwm option was given.
            if opts::get().print_pwm {
                println!("{:?} {:?}", $label, time);
            }
        }
    );
);

pub struct PaintTimeMetrics {
    navigation_start: RwLock<Option<f64>>,
    first_paint: RwLock<Option<f64>>,
    first_content_paint: RwLock<Option<f64>>,
    time_profiler_chan: Arc<Mutex<ProfilerChan>>,
}

impl PaintTimeMetrics {
    pub fn new(time_profiler_chan: ProfilerChan)
        -> PaintTimeMetrics {
        PaintTimeMetrics {
            navigation_start: RwLock::new(None),
            first_paint: RwLock::new(None),
            first_content_paint: RwLock::new(None),
            time_profiler_chan: Arc::new(Mutex::new(time_profiler_chan)),
        }
    }

    pub fn set_navigation_start(&self, time: f64) {
        let mut navigation_start = self.navigation_start.write().unwrap();
        *navigation_start = Some(time);
    }

    make_time_setter!(first_paint, set_first_paint,
                      TimeToFirstPaint,
                      "first-paint");
    make_time_setter!(first_content_paint, set_first_content_paint,
                      TimeToFirstContentfulPaint,
                      "first-contentful-paint");

    pub fn maybe_set_first_paint<T>(&self, profiler_metadata_factory: T)
        where T: ProfilerMetadataFactory {
        {
            let attr = self.first_paint.read().unwrap();
            if attr.is_some() {
                return;
            }
        }

        self.set_first_paint(profiler_metadata_factory);
    }

    pub fn maybe_set_first_content_paint<T>(&self, profiler_metadata_factory: T,
                                         display_list: &DisplayList)
        where T: ProfilerMetadataFactory {
        {
            let attr = self.first_content_paint.read().unwrap();
            if attr.is_some() {
                return;
            }
        }

        // Analyze display list to figure out if this is the first contentful
        // paint (i.e. the display list contains items of type text, image,
        // non-white canvas or SVG)
        // TODO: SVG
        for item in &display_list.list {
            match item {
                // Canvas data is transformed to DisplayItem::Image or DisplayItem::WebGL
                &DisplayItem::Text(_) |
                &DisplayItem::Image(_) |
                &DisplayItem::WebGL(_) => {
                    self.set_first_content_paint(profiler_metadata_factory);
                    return;
                },
                _ => (),
            }
        }
    }
}
