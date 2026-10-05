/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use fft_convolver::FFTConvolver;
use log::{error, warn};
use malloc_size_of_derive::MallocSizeOf;

use crate::audio_node::{
    AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo, ChannelInterpretation,
};
use crate::block::{Block, Chunk, FRAMES_PER_BLOCK_USIZE};
use crate::buffer_source_node::AudioBuffer;

#[derive(Clone, Debug, MallocSizeOf)]
pub struct ConvolverNodeOptions {
    pub buffer: Option<AudioBuffer>,
    pub normalize: bool,
}

#[derive(Clone, Debug, MallocSizeOf)]
pub enum ConvolverNodeMessage {
    SetBuffer(#[conditional_malloc_size_of] Option<Arc<AudioBuffer>>),
    SetNormalize(bool),
}

#[derive(AudioNodeCommon)]
pub(crate) struct ConvolverNode {
    channel_info: ChannelInfo,
    buffer: Option<Arc<AudioBuffer>>,
    normalize: bool,
    convolvers: Option<Vec<FFTConvolver<f32>>>,
}

// <https://webaudio.github.io/web-audio-api/#dom-convolvernode-normalize>
fn calculate_normalization_scale(buffer: &AudioBuffer) -> f64 {
    let gain_calibration = 0.00125_f64;
    let gain_calibration_sample_rate = 44100_f64;
    let min_power = 0.000125_f64;
    // Normalize by RMS power.
    let number_of_channels = buffer.chans() as f64;
    let buffer_length = buffer.len() as f64;

    let mut power = buffer.buffers.iter().fold(0_f64, |power, channel| {
        power +
            channel.iter().fold(0_f64, |channel_power, sample| {
                channel_power + sample.powi(2) as f64
            })
    });

    power = (power / (number_of_channels * buffer_length)).sqrt();
    if power.is_infinite() {
        power = min_power;
    }
    power = power.max(min_power);
    let mut scale = 1.0 / power;
    // Calibrate to make perceived volume same as unprocessed.
    scale *= gain_calibration;
    // Scale depends on sample-rate.
    scale *= gain_calibration_sample_rate / buffer.sample_rate as f64;
    // True-stereo compensation.
    if number_of_channels == 4.0 {
        scale *= 0.5;
    }
    scale
}

fn initialize_convolvers(buffer: Arc<AudioBuffer>) -> Option<Vec<FFTConvolver<f32>>> {
    let initialize_convolver = |impulse_response: &Vec<f32>| {
        let mut convolver = FFTConvolver::<f32>::default();
        convolver
            .init(FRAMES_PER_BLOCK_USIZE * 8, impulse_response.as_slice())
            .inspect_err(|error| error!("Failed to initialize convolver {}", error))
            .ok()?;
        Some(convolver)
    };
    let mut convolvers = buffer
        .buffers
        .iter()
        .map(initialize_convolver)
        .collect::<Option<Vec<_>>>();
    // If we have a mono IR, need to create two convolvers.
    // This is to ensure we can handle the stereo input case, since the FFT Convolver
    // assumes inputs are blocks of a long-running sample.
    if let Some(convolvers) = &mut convolvers &&
        convolvers.len() == 1
    {
        let impulse_response = buffer.buffers.first()?;
        let convolver = initialize_convolver(impulse_response)?;
        convolvers.push(convolver);
    }
    convolvers
}

/// Calculates the convolution between the input and the impulse response of the convolver.
/// FFTConvolver uses the overlap-add algorithm to calculate the convolution.
// TODO: Switch to ThreadedFFTConvolver when this feature is stable.
fn linear_convolution(input: &[f32], convolver: &mut FFTConvolver<f32>) -> Option<Vec<f32>> {
    let mut output = vec![0.0; FRAMES_PER_BLOCK_USIZE];
    convolver
        .process(input, output.as_mut_slice())
        .inspect_err(|e| {
            error!(
                "Linear convolution of input with impulse response failed {}",
                e
            )
        })
        .ok()?;
    Some(output)
}

fn downmix_four_channel_output_to_stereo(outputs: Vec<Vec<f32>>) -> Vec<f32> {
    if outputs.len() != 4 {
        warn!("Output does not have four channels.");
        return Vec::new();
    }
    let mut output = Vec::with_capacity(outputs[0].len() * 2);
    let mixed_output_0 = outputs[0]
        .iter()
        .zip(outputs[2].iter())
        .map(|(x, y)| x + y)
        .collect::<Vec<_>>();
    let mixed_output_1 = outputs[1]
        .iter()
        .zip(outputs[3].iter())
        .map(|(x, y)| x + y)
        .collect::<Vec<_>>();
    output.extend(mixed_output_0);
    output.extend(mixed_output_1);
    output
}

impl ConvolverNode {
    pub fn new(options: ConvolverNodeOptions, channel_info: ChannelInfo) -> Self {
        let buffer = options.buffer.map(|mut buffer| {
            if options.normalize {
                let normalization_scale = calculate_normalization_scale(&buffer);
                buffer.scale(normalization_scale);
            }
            Arc::new(buffer)
        });

        let convolvers = buffer
            .as_ref()
            .and_then(|buffer| initialize_convolvers(buffer.clone()));
        Self {
            channel_info,
            buffer,
            normalize: options.normalize,
            convolvers,
        }
    }

    fn handle_convolver_message(&mut self, message: ConvolverNodeMessage, _sample_rate: f32) {
        match message {
            ConvolverNodeMessage::SetBuffer(maybe_buffer) => {
                let mut buffer = maybe_buffer;
                // > Changes to this value do not take effect until the next time the buffer attribute is set.
                // <https://webaudio.github.io/web-audio-api/#dom-convolvernode-normalize>
                if let Some(input_buffer) = buffer.as_ref() &&
                    self.normalize
                {
                    let mut normalized_buffer = (**input_buffer).clone();
                    let normalization_scale = calculate_normalization_scale(&normalized_buffer);
                    normalized_buffer.scale(normalization_scale);
                    buffer = Some(Arc::new(normalized_buffer));
                }
                self.buffer = buffer;
                // Precompute buffer FFTs.
                self.convolvers = self
                    .buffer
                    .as_ref()
                    .and_then(|buffer| initialize_convolvers(buffer.clone()));
            },
            ConvolverNodeMessage::SetNormalize(normalize) => {
                self.normalize = normalize;
            },
        }
    }
}

impl AudioNodeEngine for ConvolverNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::ConvolverNode
    }

    fn process(&mut self, inputs: Chunk, _info: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == 1);

        let Some(buffer) = self.buffer.as_ref() else {
            return Chunk::explicit_silence();
        };

        let Some(convolvers) = &mut self.convolvers else {
            return inputs;
        };

        let input_block = &inputs.blocks[0];

        // <https://webaudio.github.io/web-audio-api/#Convolution-channel-configurations>
        let convolution_output = match (input_block.chan_count(), buffer.chans()) {
            // If we have a mono impulse response, we need to calculate convolution for the
            // second convolver, which in the mono case uses the same impulse response as the first
            // convolver.
            // This is because fast convolution algorithms such as overlap-add used by the FFT Convolver
            // assume inputs are blocks of a long-running sample.
            // Results from previous blocks are used when calculating for subsequent blocks.
            // If the input increases channel count, the convolver will need the results from
            // previous blocks as if it had always had the upmixed input.
            //
            // Mono with Mono Response
            (1, 1) => {
                let output = linear_convolution(input_block.data_chan(0), &mut convolvers[0]);
                let input = match self.channel_info.interpretation {
                    ChannelInterpretation::Discrete => &[0.; FRAMES_PER_BLOCK_USIZE],
                    ChannelInterpretation::Speakers => input_block.data_chan(0),
                };
                let _ = linear_convolution(input, &mut convolvers[1]);
                output
            },
            // Stereo with Mono Response
            (2, 1) => {
                let outputs = vec![
                    linear_convolution(input_block.data_chan(0), &mut convolvers[0]),
                    linear_convolution(input_block.data_chan(1), &mut convolvers[1]),
                ]
                .into_iter()
                .collect::<Option<Vec<Vec<_>>>>();
                outputs.map(|outputs| outputs.into_iter().flatten().collect())
            },
            // Mono with Stereo Response
            (1, 2) => {
                let outputs = vec![
                    linear_convolution(input_block.data_chan(0), &mut convolvers[0]),
                    linear_convolution(input_block.data_chan(0), &mut convolvers[1]),
                ]
                .into_iter()
                .collect::<Option<Vec<Vec<_>>>>();
                outputs.map(|outputs| outputs.into_iter().flatten().collect())
            },
            // Stereo with Stereo Response
            (2, 2) => {
                let outputs = vec![
                    linear_convolution(input_block.data_chan(0), &mut convolvers[0]),
                    linear_convolution(input_block.data_chan(1), &mut convolvers[1]),
                ]
                .into_iter()
                .collect::<Option<Vec<Vec<_>>>>();
                outputs.map(|outputs| outputs.into_iter().flatten().collect())
            },
            // Stereo with "true" Stereo Matrix Response
            (2, 4) => {
                let outputs = vec![
                    linear_convolution(input_block.data_chan(0), &mut convolvers[0]),
                    linear_convolution(input_block.data_chan(0), &mut convolvers[1]),
                    linear_convolution(input_block.data_chan(1), &mut convolvers[2]),
                    linear_convolution(input_block.data_chan(1), &mut convolvers[3]),
                ]
                .into_iter()
                .collect::<Option<Vec<Vec<_>>>>();
                outputs.map(downmix_four_channel_output_to_stereo)
            },
            // Mono with Stereo Matrix Response
            (1, 4) => {
                let outputs = convolvers
                    .iter_mut()
                    .map(|convolver| linear_convolution(input_block.data_chan(0), convolver))
                    .collect::<Option<Vec<Vec<_>>>>();
                outputs.map(downmix_four_channel_output_to_stereo)
            },
            _ => {
                error!(
                    "Invalid channel configuration. Input channels: {}, Impulse Response channels: {}",
                    input_block.chan_count(),
                    buffer.chans()
                );
                return inputs;
            },
        };

        let Some(output) = convolution_output else {
            error!("Failed to calculate convolution.");
            return Chunk::explicit_silence();
        };
        let block = Block::for_vec(output);
        let mut chunk = Chunk::default();
        chunk.blocks.push(block);
        chunk
    }
    make_message_handler!(
        ConvolverNode: handle_convolver_message
    );
}
