/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;

use crate::audio_node::{
    AudioNodeEngine, AudioNodeType, AudioScheduledSourceNodeMessage, BlockInfo, ChannelInfo,
    OnEndedCallback, ShouldPlay,
};
use crate::block::{Chunk, Tick};
use crate::param::{Param, ParamType};
use crate::periodic_wave::PeriodicWave;

#[derive(Clone, Debug, MallocSizeOf)]
pub enum OscillatorType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Custom(PeriodicWave),
}

#[derive(Clone, Debug, MallocSizeOf)]
pub struct OscillatorNodeOptions {
    pub oscillator_type: OscillatorType,
    pub freq: f32,
    pub detune: f32,
}

impl Default for OscillatorNodeOptions {
    fn default() -> Self {
        OscillatorNodeOptions {
            oscillator_type: OscillatorType::Sine,
            freq: 440.,
            detune: 0.,
        }
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
pub enum OscillatorNodeMessage {
    SetOscillatorType(OscillatorType),
    SetPeriodicWave(PeriodicWave),
}

#[derive(AudioScheduledSourceNode, AudioNodeCommon)]
pub(crate) struct OscillatorNode {
    channel_info: ChannelInfo,
    periodic_wave: PeriodicWave,
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
            periodic_wave: PeriodicWave::generate_waveform_coefficients(options.oscillator_type),
            frequency: Param::new(options.freq),
            detune: Param::new(options.detune),
            phase: 0.,
            start_at: None,
            stop_at: None,
            onended_callback: None,
        }
    }

    pub fn update_parameters(&mut self, info: &BlockInfo, tick: Tick) -> bool {
        let (frequency_updated, detune_updated) = (
            self.frequency.update(info, tick),
            self.detune.update(info, tick),
        );
        frequency_updated || detune_updated
    }

    fn compute_oscillator_frequency(&self, sample_rate: f64) -> f64 {
        // Clamp params based on web audio specs
        // <https://www.w3.org/TR/webaudio-1.1/#dom-oscillatornode-detune>
        // <https://www.w3.org/TR/webaudio-1.1/#dom-oscillatornode-frequency>
        let mut detune = self.detune.value() as f64;
        let critical_detune = 1200.0 * f64::MAX.log2();
        detune = detune.clamp(-critical_detune, critical_detune);
        let nyquist = sample_rate / 2.0;
        let mut frequency = (self.frequency.value() as f64).clamp(-nyquist, nyquist);
        frequency *= (detune / 1200.0).exp2();
        // Clamp to nyquist
        frequency.clamp(-nyquist, nyquist)
    }

    fn handle_oscillator_message(&mut self, message: OscillatorNodeMessage, _sample_rate: f32) {
        match message {
            OscillatorNodeMessage::SetOscillatorType(o) => {
                self.periodic_wave = PeriodicWave::generate_waveform_coefficients(o);
            },
            OscillatorNodeMessage::SetPeriodicWave(w) => {
                self.periodic_wave = w;
            },
        }
    }
}

impl AudioNodeEngine for OscillatorNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::OscillatorNode
    }

    fn process(&mut self, mut inputs: Chunk, info: &BlockInfo) -> Chunk {
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
            let mut oscillator_frequency = self.compute_oscillator_frequency(sample_rate);
            let mut step = two_pi * oscillator_frequency / sample_rate;
            while let Some(mut frame) = iter.next() {
                let tick = frame.tick();
                if tick < start_at {
                    continue;
                } else if tick > stop_at {
                    break;
                }

                if self.update_parameters(info, tick) {
                    oscillator_frequency = self.compute_oscillator_frequency(sample_rate);
                    step = two_pi * oscillator_frequency / sample_rate;
                }
                let value = vol *
                    self.periodic_wave.calculate_waveform(
                        oscillator_frequency,
                        sample_rate,
                        self.phase,
                    ) as f32;

                frame.mutate_with(|sample, _| *sample = value);
                // Wrap phase if necessary in order to keep it in radians
                self.phase = (self.phase + step).rem_euclid(two_pi);
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
