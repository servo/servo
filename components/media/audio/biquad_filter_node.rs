use std::f64::consts::{PI, SQRT_2};

use smallvec::SmallVec;

use crate::block::{Chunk, Tick};
use crate::node::{AudioNodeEngine, AudioNodeMessage, AudioNodeType, BlockInfo, ChannelInfo};
use crate::param::{Param, ParamType};

#[derive(Copy, Clone, Debug)]
pub struct BiquadFilterNodeOptions {
    pub filter: FilterType,
    pub frequency: f32,
    pub detune: f32,
    pub q: f32,
    pub gain: f32,
}

#[derive(Copy, Clone, Debug)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    LowShelf,
    HighShelf,
    Peaking,
    Notch,
    AllPass,
}

impl Default for BiquadFilterNodeOptions {
    fn default() -> Self {
        BiquadFilterNodeOptions {
            filter: FilterType::LowPass,
            frequency: 350.,
            detune: 0.,
            q: 1.,
            gain: 0.,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BiquadFilterNodeMessage {
    SetFilterType(FilterType),
}

/// The last two input and output values, per-channel
// Default sets all fields to zero
#[derive(Default, Copy, Clone, PartialEq)]
struct BiquadState {
    /// The input value from last frame
    x1: f64,
    /// The input value from two frames ago
    x2: f64,
    /// The output value from last frame
    y1: f64,
    /// The output value from two frames ago
    y2: f64,
}

impl BiquadState {
    /// Update with new input/output values from this frame
    fn update(&mut self, x: f64, y: f64) {
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
    }
}

/// https://webaudio.github.io/web-audio-api/#biquadfilternode
#[derive(AudioNodeCommon)]
pub(crate) struct BiquadFilterNode {
    channel_info: ChannelInfo,
    filter: FilterType,
    frequency: Param,
    detune: Param,
    q: Param,
    gain: Param,
    /// The computed filter parameter b0
    /// This is actually b0 / a0, we pre-divide
    /// for efficiency
    b0: f64,
    /// The computed filter parameter b1
    /// This is actually b1 / a0, we pre-divide
    /// for efficiency
    b1: f64,
    /// The computed filter parameter b2
    /// This is actually b2 / a0, we pre-divide
    /// for efficiency
    b2: f64,
    /// The computed filter parameter a1
    /// This is actually a1 / a0, we pre-divide
    /// for efficiency
    a1: f64,
    /// The computed filter parameter a2
    /// This is actually a2 / a0, we pre-divide
    /// for efficiency
    a2: f64,
    /// Stored filter state, this contains the last two
    /// frames of input and output values for every
    /// channel
    state: SmallVec<[BiquadState; 2]>,
}

impl BiquadFilterNode {
    pub fn new(
        options: BiquadFilterNodeOptions,
        channel_info: ChannelInfo,
        sample_rate: f32,
    ) -> Self {
        let mut ret = Self {
            channel_info,
            filter: options.filter,
            frequency: Param::new(options.frequency),
            gain: Param::new(options.gain),
            q: Param::new(options.q),
            detune: Param::new(options.detune),
            b0: 0.,
            b1: 0.,
            b2: 0.,
            a1: 0.,
            a2: 0.,
            state: SmallVec::new(),
        };
        ret.update_coefficients(sample_rate);
        ret
    }

    pub fn update_parameters(&mut self, info: &BlockInfo, tick: Tick) -> bool {
        let mut changed = self.frequency.update(info, tick);
        changed |= self.detune.update(info, tick);
        changed |= self.q.update(info, tick);
        changed |= self.gain.update(info, tick);

        if changed {
            self.update_coefficients(info.sample_rate);
        }
        changed
    }

    /// Set to the constant z-transform y[n] = b0 * x[n]
    fn constant_z_transform(&mut self, b0: f64) {
        self.b0 = b0;
        self.b1 = 0.;
        self.b2 = 0.;
        self.a1 = 0.;
        self.a2 = 0.;
    }

    /// Update the coefficients a1, a2, b0, b1, b2, given the sample_rate
    ///
    /// See https://webaudio.github.io/web-audio-api/#filters-characteristics
    fn update_coefficients(&mut self, fs: f32) {
        let g: f64 = self.gain.value().into();
        let q: f64 = self.q.value().into();
        let freq: f64 = self.frequency.value().into();
        let f0: f64 = freq * (2.0_f64).powf(self.detune.value() as f64 / 1200.);
        let fs: f64 = fs.into();
        // clamp to nominal range
        // https://webaudio.github.io/web-audio-api/#biquadfilternode
        let f0 = if f0 > fs / 2. || !f0.is_finite() {
            fs / 2.
        } else if f0 < 0. {
            0.
        } else {
            f0
        };

        let normalized = f0 / fs;
        let a = 10.0_f64.powf(g / 40.);

        // the boundary values sometimes need limits to
        // be taken
        match self.filter {
            FilterType::LowPass => {
                if normalized == 1. {
                    self.constant_z_transform(1.);
                    return;
                } else if normalized == 0. {
                    self.constant_z_transform(0.);
                    return;
                }
            },
            FilterType::HighPass => {
                if normalized == 1. {
                    self.constant_z_transform(0.);
                    return;
                } else if normalized == 0. {
                    self.constant_z_transform(1.);
                    return;
                }
            },
            FilterType::LowShelf => {
                if normalized == 1. {
                    self.constant_z_transform(a * a);
                    return;
                } else if normalized == 0. {
                    self.constant_z_transform(1.);
                    return;
                }
            },
            FilterType::HighShelf => {
                if normalized == 1. {
                    self.constant_z_transform(1.);
                    return;
                } else if normalized == 0. {
                    self.constant_z_transform(a * a);
                    return;
                }
            },
            FilterType::Peaking => {
                if normalized == 0. || normalized == 1. {
                    self.constant_z_transform(1.);
                    return;
                } else if q <= 0. {
                    self.constant_z_transform(a * a);
                    return;
                }
            },
            FilterType::AllPass => {
                if normalized == 0. || normalized == 1. {
                    self.constant_z_transform(1.);
                    return;
                } else if q <= 0. {
                    self.constant_z_transform(-1.);
                    return;
                }
            },
            FilterType::Notch => {
                if normalized == 0. || normalized == 1. {
                    self.constant_z_transform(1.);
                    return;
                } else if q <= 0. {
                    self.constant_z_transform(0.);
                    return;
                }
            },
            FilterType::BandPass => {
                if normalized == 0. || normalized == 1. {
                    self.constant_z_transform(0.);
                    return;
                } else if q <= 0. {
                    self.constant_z_transform(1.);
                    return;
                }
            },
        }

        let omega0 = 2. * PI * normalized;
        let sin_omega = omega0.sin();
        let cos_omega = omega0.cos();
        let alpha_q = sin_omega / (2. * q);
        let alpha_q_db = sin_omega / (2. * 10.0_f64.powf(q / 20.));
        let alpha_s = sin_omega / SQRT_2;

        // we predivide by a0
        let a0;

        match self.filter {
            FilterType::LowPass => {
                self.b0 = (1. - cos_omega) / 2.;
                self.b1 = 1. - cos_omega;
                self.b2 = self.b1 / 2.;
                a0 = 1. + alpha_q_db;
                self.a1 = -2. * cos_omega;
                self.a2 = 1. - alpha_q_db;
            },
            FilterType::HighPass => {
                self.b0 = (1. + cos_omega) / 2.;
                self.b1 = -(1. + cos_omega);
                self.b2 = -self.b1 / 2.;
                a0 = 1. + alpha_q_db;
                self.a1 = -2. * cos_omega;
                self.a2 = 1. - alpha_q_db;
            },
            FilterType::BandPass => {
                self.b0 = alpha_q;
                self.b1 = 0.;
                self.b2 = -alpha_q;
                a0 = 1. + alpha_q;
                self.a1 = -2. * cos_omega;
                self.a2 = 1. - alpha_q;
            },
            FilterType::Notch => {
                self.b0 = 1.;
                self.b1 = -2. * cos_omega;
                self.b2 = 1.;
                a0 = 1. + alpha_q;
                self.a1 = -2. * cos_omega;
                self.a2 = 1. - alpha_q;
            },
            FilterType::AllPass => {
                self.b0 = 1. - alpha_q;
                self.b1 = -2. * cos_omega;
                self.b2 = 1. + alpha_q;
                a0 = 1. + alpha_q;
                self.a1 = -2. * cos_omega;
                self.a2 = 1. - alpha_q;
            },
            FilterType::Peaking => {
                self.b0 = 1. + alpha_q * a;
                self.b1 = -2. * cos_omega;
                self.b2 = 1. - alpha_q * a;
                a0 = 1. + alpha_q / a;
                self.a1 = -2. * cos_omega;
                self.a2 = 1. - alpha_q / a;
            },
            FilterType::LowShelf => {
                let alpha_rt_a = 2. * alpha_s * a.sqrt();
                self.b0 = a * ((a + 1.) - (a - 1.) * cos_omega + alpha_rt_a);
                self.b1 = 2. * a * ((a - 1.) - (a + 1.) * cos_omega);
                self.b2 = a * ((a + 1.) - (a - 1.) * cos_omega - alpha_rt_a);
                a0 = (a + 1.) + (a - 1.) * cos_omega + alpha_rt_a;
                self.a1 = -2. * ((a - 1.) + (a + 1.) * cos_omega);
                self.a2 = (a + 1.) + (a - 1.) * cos_omega - alpha_rt_a;
            },
            FilterType::HighShelf => {
                let alpha_rt_a = 2. * alpha_s * a.sqrt();
                self.b0 = a * ((a + 1.) + (a - 1.) * cos_omega + alpha_rt_a);
                self.b1 = -2. * a * ((a - 1.) + (a + 1.) * cos_omega);
                self.b2 = a * ((a + 1.) + (a - 1.) * cos_omega - alpha_rt_a);
                a0 = (a + 1.) - (a - 1.) * cos_omega + alpha_rt_a;
                self.a1 = 2. * ((a - 1.) - (a + 1.) * cos_omega);
                self.a2 = (a + 1.) - (a - 1.) * cos_omega - alpha_rt_a;
            },
        }
        self.b0 = self.b0 / a0;
        self.b1 = self.b1 / a0;
        self.b2 = self.b2 / a0;
        self.a1 = self.a1 / a0;
        self.a2 = self.a2 / a0;
    }
}

impl AudioNodeEngine for BiquadFilterNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::BiquadFilterNode
    }

    fn process(&mut self, mut inputs: Chunk, info: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == 1);
        self.state
            .resize(inputs.blocks[0].chan_count() as usize, Default::default());
        self.update_parameters(info, Tick(0));

        // XXXManishearth this node has tail time, so even if the block is silence
        // we must still compute things on it. However, it is possible to become
        // a dumb passthrough as long as we reach a quiescent state
        //
        // see https://dxr.mozilla.org/mozilla-central/rev/87a95e1b7ec691bef7b938e722fe1b01cce68664/dom/media/webaudio/blink/Biquad.cpp#81-91

        let repeat_or_silence = inputs.blocks[0].is_silence() || inputs.blocks[0].is_repeat();

        if repeat_or_silence && !self.state.iter().all(|s| *s == self.state[0]) {
            // In case our input is repeat/silence but our states are not identical, we must
            // explicitly duplicate, since mutate_with will otherwise only operate
            // on the first channel, ignoring the states of the later ones
            inputs.blocks[0].explicit_repeat();
        } else {
            // In case the states are identical, just make any silence explicit,
            // since mutate_with can't handle silent blocks
            inputs.blocks[0].explicit_silence();
        }

        {
            let mut iter = inputs.blocks[0].iter();
            while let Some(mut frame) = iter.next() {
                self.update_parameters(info, frame.tick());
                frame.mutate_with(|sample, chan| {
                    let state = &mut self.state[chan as usize];
                    let x0 = *sample as f64;
                    let y0 = self.b0 * x0 + self.b1 * state.x1 + self.b2 * state.x2 -
                        self.a1 * state.y1 -
                        self.a2 * state.y2;
                    *sample = y0 as f32;
                    state.update(x0, y0);
                });
            }
        }

        if inputs.blocks[0].is_repeat() {
            let state = self.state[0];
            self.state.iter_mut().for_each(|s| *s = state);
        }

        inputs
    }

    fn get_param(&mut self, id: ParamType) -> &mut Param {
        match id {
            ParamType::Frequency => &mut self.frequency,
            ParamType::Detune => &mut self.detune,
            ParamType::Q => &mut self.q,
            ParamType::Gain => &mut self.gain,
            _ => panic!("Unknown param {:?} for BiquadFilterNode", id),
        }
    }

    fn message_specific(&mut self, message: AudioNodeMessage, sample_rate: f32) {
        match message {
            AudioNodeMessage::BiquadFilterNode(m) => match m {
                BiquadFilterNodeMessage::SetFilterType(f) => {
                    self.filter = f;
                    self.update_coefficients(sample_rate);
                },
            },
            _ => (),
        }
    }
}
