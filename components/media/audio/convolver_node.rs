/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use fft_convolver::FFTConvolver;
use log::error;
use malloc_size_of_derive::MallocSizeOf;
use num_complex::Complex64;
use realfft::RealFftPlanner;
use crate::{audio_node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo}, block::{Block, Chunk, FRAMES_PER_BLOCK_USIZE}, buffer_source_node::AudioBuffer};

#[derive(Clone, Debug, MallocSizeOf)]
pub struct ConvolverNodeOptions {
    pub buffer: Option<AudioBuffer>,
    pub normalize: bool,
}

#[derive(Clone, Debug, MallocSizeOf)]
pub enum ConvolverNodeMessage {
    SetBuffer(Option<AudioBuffer>),
    SetNormalize(bool),
}

#[derive(AudioNodeCommon)]
pub(crate) struct ConvolverNode {
    channel_info: ChannelInfo,
    buffer: Option<AudioBuffer>,
    normalize: bool,
    normalization_scale: Option<f64>,
    convolvers: Option<Vec<FFTConvolver<f32>>>,
}

fn calculate_normalization_scale(buffer: &Option<AudioBuffer>, normalize: bool) -> Option<f64> {
    if normalize {
        buffer.as_ref().map(|buffer|{

    let gain_calibration = 0.00125_f64;
    let gain_calibration_sample_rate = 44100_f64;
    let min_power = 0.000125_f64;
    // Normalize by RMS power.
    let number_of_channels = buffer.chans() as f64;
    let buffer_length = buffer.len() as f64;
    
    let mut power = buffer.buffers.iter().fold(0_f64, |power, channel| {
        power + channel.iter().fold(0_f64, |channel_power, sample| channel_power + sample.powi(2) as f64)
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
        })
    } else {
        None
    }
}

fn initialize_convolvers(buffer: Option<&AudioBuffer>) -> Option<Vec<FFTConvolver<f32>>> {
    let buffer = buffer?;
    let mut convolvers = buffer.buffers.iter().map(|impulse_response| {
        let mut convolver = FFTConvolver::<f32>::default();
        convolver.init(FRAMES_PER_BLOCK_USIZE * 2, impulse_response.as_slice()).inspect_err(|e| error!("Failed to initialize convolver {}", e)).ok()?;
        Some(convolver)
    }).collect::<Option<Vec<_>>>();
    // If we have a mono IR, need to create two convolvers.
    // This is to ensure we can handle the stereo input chanse, since the FFT Convolver 
    // assumes inputs are blocks of a long-running sample.
    if let Some(convolvers) = &mut convolvers {
        if convolvers.len() == 1 {
            convolvers.push(convolvers[0].clone());
        }
    }
    convolvers
}

fn linear_convolution(input: &[f32], convolver: &mut FFTConvolver<f32>) -> Option<Vec<f32>> {
    let mut output = vec![0.0; FRAMES_PER_BLOCK_USIZE];
    convolver.process(input, output.as_mut_slice()).inspect_err(|e| error!("Linear convolution of input with impulse response failed {}", e)).ok()?;
    Some(output)
}

impl ConvolverNode {
    pub fn new(options: ConvolverNodeOptions, channel_info: ChannelInfo) -> Self {
        let normalization_scale = calculate_normalization_scale(&options.buffer, options.normalize);
        let convolvers = initialize_convolvers(options.buffer.as_ref());
        Self {
            channel_info,
            buffer: options.buffer,
            normalize: options.normalize,
            normalization_scale,
            convolvers,
        }
    }

    fn handle_convolver_message(&mut self, message: ConvolverNodeMessage, _sample_rate: f32) {
        match message {
            ConvolverNodeMessage::SetBuffer(buffer) => {
                self.buffer = buffer;
                self.normalization_scale = calculate_normalization_scale(&self.buffer, self.normalize);
                // Precompute buffer FFTs
                self.convolvers = initialize_convolvers(self.buffer.as_ref());
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
            return inputs;
        };

        let Some(convolvers) = &mut self.convolvers else {
            return inputs;
        };

        let input_block = &inputs.blocks[0];
        let mut convolution_output = match (input_block.chan_count(), buffer.chans()) {
            // Mono with Mono
            (1, 1) => {
                linear_convolution(input_block.data_chan(0), &mut convolvers[0]).unwrap_or_default()
            }
            // Mono with Stereo Response
            (1, 2) => {
                let output_0 = linear_convolution(input_block.data_chan(0), &mut convolvers[0]).unwrap_or_default();
                let output_1 = linear_convolution(input_block.data_chan(0), &mut convolvers[1]).unwrap_or_default();

                let mut output = Vec::with_capacity(output_0.len() * 2);
                output.extend(output_0);
                output.extend(output_1);
                output
            }
            // Stereo with Mono Response
            (2, 1) => {
                let output_0 = linear_convolution(input_block.data_chan(0), &mut convolvers[0]).unwrap_or_default();
                let output_1 = linear_convolution(input_block.data_chan(1), &mut convolvers[1]).unwrap_or_default();

                let mut output = Vec::with_capacity(output_0.len() * 2);
                output.extend(output_0);
                output.extend(output_1);
                output
            }
            // Stereo with Stereo Response
            (2, 2) => {
                let output_0 = linear_convolution(input_block.data_chan(0), &mut convolvers[0]).unwrap_or_default();
                let output_1 = linear_convolution(input_block.data_chan(1), &mut convolvers[1]).unwrap_or_default();

                let mut output = Vec::with_capacity(output_0.len() * 2);
                output.extend(output_0);
                output.extend(output_1);
                output
            }
            // Stereo with "true" Stereo Matrix Response
            (2, 4) => {
                let output_0 = linear_convolution(input_block.data_chan(0), &mut convolvers[0]).unwrap_or_default();
                let output_1 = linear_convolution(input_block.data_chan(0), &mut convolvers[1]).unwrap_or_default();
                let output_2 = linear_convolution(input_block.data_chan(1), &mut convolvers[2]).unwrap_or_default();
                let output_3 = linear_convolution(input_block.data_chan(1), &mut convolvers[3]).unwrap_or_default();
                let mut output = Vec::with_capacity(output_0.len() * 4);
                output.extend(output_0);
                output.extend(output_1);
                output.extend(output_2);
                output.extend(output_3);
                output
            }
            // Mono with Stereo Matrix Response
            (1, 4) => {
                let output_0 = linear_convolution(input_block.data_chan(0), &mut convolvers[0]).unwrap_or_default();
                let output_1 = linear_convolution(input_block.data_chan(0), &mut convolvers[1]).unwrap_or_default();
                let output_2 = linear_convolution(input_block.data_chan(1), &mut convolvers[2]).unwrap_or_default();
                let output_3 = linear_convolution(input_block.data_chan(1), &mut convolvers[3]).unwrap_or_default();
                let mut output = Vec::with_capacity(output_0.len() * 4);
                output.extend(output_0);
                output.extend(output_1);
                output.extend(output_2);
                output.extend(output_3);
                output
            }
            _ => {
                error!("Invalid channel configuration. Input channels: {}, Impulse Response channels: {}", input_block.chan_count(), buffer.chans());
                return inputs;
            }
        };
        
        if self.normalize {
            // Take this calculated normalizationScale value and
            // multiply it by the result of the linear convolution resulting from processing 
            // the input with the impulse response (represented by the buffer) to produce the final output.
            if let Some(normalization_scale) = self.normalization_scale {
                convolution_output = convolution_output.into_iter().map(|x| (x as f64 * normalization_scale) as f32).collect();
            }
        }
        let block = Block::for_vec(convolution_output);
        let mut chunk = Chunk::default();
        chunk.blocks.push(block);
        chunk
    }
make_message_handler!(
        ConvolverNode: handle_convolver_message
    );
}
