/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


#[cfg(feature = "energy-profiling")]
pub fn read_energy_uj() -> u64 {
    energymon::read_energy_uj()
}

#[cfg(not(feature = "energy-profiling"))]
pub fn read_energy_uj() -> u64 {
    0
}

#[cfg(feature = "energy-profiling")]
pub fn energy_interval_ms() -> u32 {
    energymon::get_min_interval_ms()
}

#[cfg(not(feature = "energy-profiling"))]
pub fn energy_interval_ms() -> u32 {
    1000
}

#[cfg(feature = "energy-profiling")]
mod energymon {
    extern crate energymon;
    extern crate energy_monitor;

    use self::energy_monitor::EnergyMonitor;
    use self::energymon::EnergyMon;
    use std::mem;
    use std::sync::{Once, ONCE_INIT};


    static mut EM: Option<*mut EnergyMon> = None;

    /// Read energy from the energy monitor, otherwise return 0.
    pub fn read_energy_uj() -> u64 {
        static ONCE: Once = ONCE_INIT;
        ONCE.call_once(|| {
            if let Ok(em) = EnergyMon::new() {
                println!("Started energy monitoring from: {}", em.source());
                unsafe {
                    EM = Some(mem::transmute(Box::new(em)));
                }
            }
        });

        unsafe {
            // EnergyMon implementations of EnergyMonitor always return a value
            EM.map_or(0, |em| (*em).read_uj().unwrap())
        }
    }

    pub fn get_min_interval_ms() -> u32 {
        unsafe {
            EM.map_or(0, |em| ((*em).interval_us() as f64 / 1000.0).ceil() as u32)
        }
    }

}
