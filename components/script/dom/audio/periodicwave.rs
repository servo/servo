/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::HandleObject;
use script_bindings::codegen::GenericBindings::PeriodicWaveBinding::PeriodicWaveMethods;
use script_bindings::num::Finite;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use servo_media::audio::periodic_wave::{
    PeriodicWave as ServoMediaPeriodicWave, PeriodicWaveOptions as ServoMediaPeriodicWaveOptions,
};

use crate::conversions::Convert;
use crate::dom::audio::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::codegen::Bindings::PeriodicWaveBinding::PeriodicWaveOptions;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct PeriodicWave {
    reflector_: Reflector,
    context: Dom<BaseAudioContext>,
    /// <https://webaudio.github.io/web-audio-api/#dom-periodicwave-imag-slot>
    imag: Vec<Finite<f32>>,
    /// <https://webaudio.github.io/web-audio-api/#dom-periodicwave-real-slot>
    real: Vec<Finite<f32>>,
    /// <https://webaudio.github.io/web-audio-api/#dom-periodicwave-imag-slot>
    normalize: bool,
}

impl PeriodicWaveMethods<crate::DomTypeHolder> for PeriodicWave {
    /// <https://webaudio.github.io/web-audio-api/#dom-periodicwave-periodicwave>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        context: &BaseAudioContext,
        options: &PeriodicWaveOptions,
    ) -> Fallible<DomRoot<PeriodicWave>> {
        let (real, imag) = match (options.real.as_ref(), options.imag.as_ref()) {
            // If both options.real and options.imag are present
            (Some(real), Some(imag)) => {
                let mut real = real.to_vec();
                let mut imag = imag.to_vec();
                // If the lengths of options.real and options.imag are different or if either length is less than 2,
                // throw an IndexSizeError and abort this algorithm
                if real.len() != imag.len() {
                    return Err(Error::IndexSize(Some(String::from(
                        "real and imag coefficients have different lengths",
                    ))));
                }
                if real.len() < 2 || imag.len() < 2 {
                    return Err(Error::IndexSize(Some(String::from(
                        "At least one of real or imag coefficients have length less than 2",
                    ))));
                }
                // Set the DC component to 0
                real[0] = Finite::wrap(0.0);
                imag[0] = Finite::wrap(0.0);
                (real, imag)
            },
            // If only options.real is present
            (Some(real), None) => {
                let mut real = real.to_vec();
                // If length of options.real is less than 2, throw an IndexSizeError and abort this algorithm
                if real.len() < 2 {
                    return Err(Error::IndexSize(Some(String::from(
                        "real coefficients have length less than 2",
                    ))));
                }
                // Set [[real]] and [[imag]] to arrays with the same length as options.real
                let real_len = real.len();
                // Set the DC component to 0
                real[0] = Finite::wrap(0.0);
                // set [[imag]] to all zeros
                (real, vec![Finite::wrap(0.0); real_len])
            },
            // If only options.imag is present
            (None, Some(imag)) => {
                let mut imag = imag.to_vec();
                // If length of options.imag is less than 2, throw an IndexSizeError and abort this algorithm
                if imag.len() < 2 {
                    return Err(Error::IndexSize(Some(String::from(
                        "imag coefficients have length less than 2",
                    ))));
                }
                // Set [[real]] and [[imag]] to arrays with the same length as options.imag
                let imag_len = imag.len();
                // Set the DC component to 0
                imag[0] = Finite::wrap(0.0);
                // set [[real]] to all zeros
                (vec![Finite::wrap(0.0); imag_len], imag)
            },
            (None, None) => (
                // Set [[real]] and [[imag]] to zero-filled arrays of length 2
                vec![Finite::wrap(0.0); 2],
                // Set element at index 1 of [[imag]] to 1
                vec![Finite::wrap(0.0), Finite::wrap(1.0)],
            ),
        };
        Ok(reflect_dom_object_with_proto(
            cx,
            Box::new(PeriodicWave {
                reflector_: Reflector::new(),
                context: Dom::from_ref(context),
                real,
                imag,
                // Initialize [[normalize]] to the inverse of the disableNormalization attribute of
                // the PeriodicWaveConstraints on the PeriodicWaveOptions
                normalize: !options.parent.disableNormalization,
            }),
            window,
            proto,
        ))
    }
}

impl Convert<ServoMediaPeriodicWave> for &PeriodicWave {
    fn convert(self) -> ServoMediaPeriodicWave {
        ServoMediaPeriodicWave::new(ServoMediaPeriodicWaveOptions::new(
            self.imag.iter().map(|x| **x).collect(),
            self.real.iter().map(|x| **x).collect(),
            !self.normalize,
        ))
    }
}
