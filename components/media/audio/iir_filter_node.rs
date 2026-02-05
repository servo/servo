use std::collections::VecDeque;
use std::sync::Arc;

use log::warn;
use num_complex::Complex64;

use crate::block::Chunk;
use crate::node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo};

const MAX_COEFFS: usize = 20;

#[derive(Debug)]
pub struct IIRFilterNodeOptions {
    pub feedforward: Arc<Vec<f64>>,
    pub feedback: Arc<Vec<f64>>,
}

#[derive(Clone)]
struct IIRFilter {
    feedforward: Arc<Vec<f64>>,
    feedback: Arc<Vec<f64>>,
    inputs: VecDeque<f64>,
    outputs: VecDeque<f64>,
}

impl IIRFilter {
    fn new(feedforward: Arc<Vec<f64>>, feedback: Arc<Vec<f64>>) -> Self {
        Self {
            feedforward,
            feedback,
            inputs: VecDeque::with_capacity(MAX_COEFFS),
            outputs: VecDeque::with_capacity(MAX_COEFFS),
        }
    }

    fn calculate_output(&mut self, input: f32) -> f32 {
        self.inputs.push_front(input as f64);

        if self.inputs.len() > MAX_COEFFS {
            self.inputs.pop_back();
        }

        let inputs_sum = self
            .feedforward
            .iter()
            .zip(self.inputs.iter())
            .fold(0.0, |acc, (c, v)| acc + c * v);

        let outputs_sum = self
            .feedback
            .iter()
            .skip(1)
            .zip(self.outputs.iter())
            .fold(0.0, |acc, (c, v)| acc + c * v);

        let output = (inputs_sum - outputs_sum) / self.feedback[0];

        if output.is_nan() {
            // Per spec:
            // Note: The UA may produce a warning to notify the user that NaN values have occurred in the filter state.
            // This is usually indicative of an unstable filter.
            //
            // But idk how to produce warnings
            warn!("NaN in IIRFilter state");
        }

        self.outputs.push_front(output);

        if self.outputs.len() > MAX_COEFFS {
            self.outputs.pop_back();
        }

        output as f32
    }
}

#[derive(AudioNodeCommon)]
pub struct IIRFilterNode {
    channel_info: ChannelInfo,
    filters: Vec<IIRFilter>,
}

impl IIRFilterNode {
    pub fn new(options: IIRFilterNodeOptions, channel_info: ChannelInfo) -> Self {
        debug_assert!(
            options.feedforward.len() > 0,
            "NotSupportedError: feedforward must have at least one coeff"
        );

        debug_assert!(
            options.feedforward.len() <= MAX_COEFFS,
            "NotSupportedError: feedforward max length is {}",
            MAX_COEFFS
        );

        debug_assert!(
            options.feedforward.iter().any(|&v| v != 0.0_f64),
            "InvalidStateError: all coeffs are zero"
        );

        debug_assert!(
            options.feedback.len() > 0,
            "NotSupportedError: feedback must have at least one coeff"
        );

        debug_assert!(
            options.feedback.len() <= MAX_COEFFS,
            "NotSupportedError: feedback max length is {}",
            MAX_COEFFS
        );

        debug_assert!(
            options.feedback[0] != 0.0,
            "InvalidStateError: first feedback coeff must not be zero"
        );

        let filter = IIRFilter::new(options.feedforward.clone(), options.feedback.clone());

        Self {
            filters: vec![filter; channel_info.computed_number_of_channels() as usize],
            channel_info,
        }
    }

    pub fn get_frequency_response(
        feedforward: &[f64],
        feedback: &[f64],
        frequency_hz: &[f32],
        mag_response: &mut [f32],
        phase_response: &mut [f32],
    ) {
        debug_assert!(
            frequency_hz.len() == mag_response.len() && frequency_hz.len() == phase_response.len(),
            "get_frequency_response params are of different length"
        );

        frequency_hz.iter().enumerate().for_each(|(idx, &f)| {
            if f < 0.0 || f >= 1.0 {
                mag_response[idx] = std::f32::NAN;
                phase_response[idx] = std::f32::NAN;
            } else {
                let f = (-f as f64) * std::f64::consts::PI;
                let z = Complex64::new(f64::cos(f), f64::sin(f));
                let numerator = Self::sum(feedforward, z);
                let denominator = Self::sum(feedback, z);

                let response = numerator / denominator;
                mag_response[idx] = response.norm() as f32;
                phase_response[idx] = response.arg() as f32;
            }
        });
    }

    fn sum(coeffs: &[f64], z: Complex64) -> Complex64 {
        coeffs.iter().fold(Complex64::new(0.0, 0.0), |acc, &coeff| {
            acc * z + Complex64::new(coeff, 0.0)
        })
    }
}

impl AudioNodeEngine for IIRFilterNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::IIRFilterNode
    }

    fn process(&mut self, inputs: Chunk, _info: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == 1);

        let mut inputs = if inputs.blocks[0].is_silence() {
            Chunk::explicit_silence()
        } else {
            inputs
        };

        let mut iter = inputs.blocks[0].iter();

        while let Some(mut frame) = iter.next() {
            frame.mutate_with(|sample, chan_idx| {
                *sample = self.filters[chan_idx as usize].calculate_output(*sample);
            });
        }
        inputs
    }
}
