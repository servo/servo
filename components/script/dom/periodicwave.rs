/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::baseaudiocontext::BaseAudioContext;
use dom::bindings::codegen::Bindings::PeriodicWaveBinding;
use dom::bindings::codegen::Bindings::PeriodicWaveBinding::PeriodicWaveOptions;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct PeriodicWave {
    reflector_: Reflector,
}

impl PeriodicWave {
    pub fn new_inherited() -> PeriodicWave {
        PeriodicWave {
            reflector_: Reflector::new(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window) -> DomRoot<PeriodicWave> {
        let periodic_wave = PeriodicWave::new_inherited();
        reflect_dom_object(Box::new(periodic_wave), window, PeriodicWaveBinding::Wrap)
    }

    pub fn Constructor(
        window: &Window,
        _context: &BaseAudioContext,
        _options: &PeriodicWaveOptions,
    ) -> Fallible<DomRoot<PeriodicWave>> {
        // TODO.
        Ok(PeriodicWave::new(&window))
    }
}
