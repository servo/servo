/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use num_traits::cast::NumCast;

use crate::block::{Chunk, Tick};
use crate::node::{
    AudioNodeEngine, AudioNodeType, AudioScheduledSourceNodeMessage, BlockInfo, ChannelInfo,
    OnEndedCallback, ShouldPlay,
};
use crate::param::{Param, ParamType};

#[derive(Clone, Debug)]
pub struct PeriodicWaveOptions {
    // XXX https://webaudio.github.io/web-audio-api/#dictdef-periodicwaveoptions
}
#[derive(Clone, Debug)]
pub enum OscillatorType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Custom,
}

#[derive(Clone, Debug)]
pub struct OscillatorNodeOptions {
    pub oscillator_type: OscillatorType,
    pub freq: f32,
    pub detune: f32,
    pub periodic_wave_options: Option<PeriodicWaveOptions>,
}

impl Default for OscillatorNodeOptions {
    fn default() -> Self {
        OscillatorNodeOptions {
            oscillator_type: OscillatorType::Sine,
            freq: 440.,
            detune: 0.,
            periodic_wave_options: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum OscillatorNodeMessage {
    SetOscillatorType(OscillatorType),
}

#[derive(AudioScheduledSourceNode, AudioNodeCommon)]
pub(crate) struct OscillatorNode {
    channel_info: ChannelInfo,
    oscillator_type: OscillatorType,
    frequency: Param,
    detune: Param,
    phase: f64,
    /// Time at which the source should start playing.
    start_at: Option<Tick>,
    /// Time at which the source should stop playing.
    stop_at: Option<Tick>,
    /// The ended event callback.
    onended_callback: Option<OnEndedCallback>,
}

impl OscillatorNode {
    pub fn new(options: OscillatorNodeOptions, channel_info: ChannelInfo) -> Self {
        Self {
            channel_info,
            oscillator_type: options.oscillator_type,
            frequency: Param::new(options.freq),
            detune: Param::new(options.detune),
            phase: 0.,
            start_at: None,
            stop_at: None,
            onended_callback: None,
        }
    }

    pub fn update_parameters(&mut self, info: &BlockInfo, tick: Tick) -> bool {
        self.frequency.update(info, tick)
    }

    fn handle_oscillator_message(&mut self, message: OscillatorNodeMessage, _sample_rate: f32) {
        match message {
            OscillatorNodeMessage::SetOscillatorType(o) => {
                self.oscillator_type = o;
            },
        }
    }
}

impl AudioNodeEngine for OscillatorNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::OscillatorNode
    }

    fn process(&mut self, mut inputs: Chunk, info: &BlockInfo) -> Chunk {
        // XXX Implement this properly and according to self.options
        // as defined in https://webaudio.github.io/web-audio-api/#oscillatornode
        use std::f64::consts::PI;
        debug_assert!(inputs.is_empty());
        inputs.blocks.push(Default::default());
        let (start_at, stop_at) = match self.should_play_at(info.frame) {
            ShouldPlay::No => {
                return inputs;
            },
            ShouldPlay::Between(start, end) => (start, end),
        };

        {
            inputs.blocks[0].explicit_silence();
            let mut iter = inputs.blocks[0].iter();

            // Convert all our parameters to the target type for calculations
            let vol: f32 = 1.0;
            let sample_rate = info.sample_rate as f64;
            let two_pi = 2.0 * PI;

            // We're carrying a phase with up to 2pi around instead of working
            // on the sample offset. High sample offsets cause too much inaccuracy when
            // converted to floating point numbers and then iterated over in 1-steps
            //
            // Also, if the frequency changes the phase should not
            let mut step = two_pi * self.frequency.value() as f64 / sample_rate;
            while let Some(mut frame) = iter.next() {
                let tick = frame.tick();
                if tick < start_at {
                    continue;
                } else if tick > stop_at {
                    break;
                }

                if self.update_parameters(info, tick) {
                    step = two_pi * self.frequency.value() as f64 / sample_rate;
                }
                let mut value = vol;
                match self.oscillator_type {
                    OscillatorType::Sine => {
                        value = vol * f32::sin(NumCast::from(self.phase).unwrap());
                    },

                    OscillatorType::Square => {
                        if self.phase >= PI && self.phase < two_pi {
                            value = vol * 1.0;
                        } else if self.phase > 0.0 && self.phase < PI {
                            value = -vol;
                        }
                    },

                    OscillatorType::Sawtooth => {
                        value = vol * (self.phase / (PI)) as f32;
                    },

                    OscillatorType::Triangle => {
                        if self.phase >= 0. && self.phase < PI / 2. {
                            value = vol * 2.0 * (self.phase / (PI)) as f32;
                        } else if self.phase >= PI / 2. && self.phase < PI {
                            value = vol * (1. - ((self.phase - (PI / 2.)) * (2. / PI)) as f32);
                        } else if self.phase >= PI && self.phase < (3. * PI / 2.) {
                            value = -vol * (1. - ((self.phase - (PI / 2.)) * (2. / PI)) as f32);
                        } else if self.phase >= 3. * PI / 2. && self.phase < 2. * PI {
                            value = vol * (-2.0) * (self.phase / (PI)) as f32;
                        }
                    },

                    OscillatorType::Custom => {},
                }

                frame.mutate_with(|sample, _| *sample = value);

                self.phase += step;
                if self.phase >= two_pi {
                    self.phase -= two_pi;
                }
            }
        }
        inputs
    }

    fn input_count(&self) -> u32 {
        0
    }

    fn get_param(&mut self, id: ParamType) -> &mut Param {
        match id {
            ParamType::Frequency => &mut self.frequency,
            ParamType::Detune => &mut self.detune,
            _ => panic!("Unknown param {:?} for OscillatorNode", id),
        }
    }
    make_message_handler!(
        AudioScheduledSourceNode: handle_source_node_message,
        OscillatorNode: handle_oscillator_message
    );
}
