/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::{HangAnnotation, MonitoredComponentType, MonitoredComponentMsg};
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::time::{Duration, Instant};


pub enum HangAlert {
    /// Report a transient hang.
    Transient(MonitoredComponentType, HangAnnotation),
    /// Report a permanent hang.
    Permanent(MonitoredComponentType, HangAnnotation),
}

impl fmt::Debug for HangAlert {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            HangAlert::Transient(ref component, ref annotation) => {
                write!(f, "Transient hang for {:?} during {:?}", component, annotation)
            },
            HangAlert::Permanent(ref component, ref annotation) => {
                write!(f, "Permanent hang for {:?} during {:?}", component, annotation)
            },
        }
    }
}

struct MonitoredComponent {
    last_activity: Cell<Instant>,
    last_annotation: Cell<Option<HangAnnotation>>,
    transient_hang_timeout: Duration,
    permanent_hang_timeout: Duration,
    is_waiting: Cell<bool>
}

#[derive(Default)]
pub struct BackgroundHangMonitor {
    monitored_components: HashMap<MonitoredComponentType, MonitoredComponent>,
    alerts: VecDeque<HangAlert>
}

impl BackgroundHangMonitor {
    pub fn handle_msg(&mut self, msg: MonitoredComponentMsg) {
        match msg {
            MonitoredComponentMsg::RegisterComponent(component_type, transient, permanent) => {
                let component = MonitoredComponent {
                    last_activity: Cell::new(Instant::now()),
                    last_annotation: Cell::new(None),
                    transient_hang_timeout: transient,
                    permanent_hang_timeout: permanent,
                    is_waiting: Cell::new(true)
                };
                assert!(self.monitored_components.insert(component_type, component).is_none(),
                        "This component was already registered for monitoring.");
            },
            MonitoredComponentMsg::NotifyActivity(component_type, annotation) => {
                let component = self.monitored_components.get(&component_type)
                    .expect("Receiced NotifyActivity for an unknown component");
                component.last_activity.set(Instant::now());
                component.last_annotation.set(Some(annotation));
                component.is_waiting.set(false);
            },
            MonitoredComponentMsg::NotifyWait(waiting) => {
                for component_type in waiting {
                    let component = self.monitored_components.get(&component_type)
                        .expect("Receiced NotifyWait for an unknown component");
                    component.last_activity.set(Instant::now());
                    component.is_waiting.set(true);
                }
            },
        }
    }

    pub fn perform_a_hang_monitor_checkpoint(&mut self) {
        for (component_type, monitored) in self.monitored_components.iter() {
            if monitored.is_waiting.get() {
                continue
            }
            let last_annotation = monitored.last_annotation.get().unwrap();
            if monitored.last_activity.get().elapsed() > monitored.permanent_hang_timeout {
                self.alerts.push_back(HangAlert::Permanent(component_type.clone(), last_annotation));
                continue
            }
            if monitored.last_activity.get().elapsed() > monitored.transient_hang_timeout {
                self.alerts.push_back(HangAlert::Transient(component_type.clone(), last_annotation));
            }
        }
    }

    pub fn collect_hang_alerts(&mut self) -> VecDeque<HangAlert> {
        self.alerts.drain(0..).collect()
    }
}
