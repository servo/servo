use speexdsp_resampler::State as SpeexResamplerState;

use crate::block::{Chunk, FRAMES_PER_BLOCK_USIZE};
use crate::node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo};

#[derive(Clone, Debug, PartialEq)]
pub enum OverSampleType {
    None,
    Double,
    Quadruple,
}

#[derive(Clone, Debug, PartialEq)]
enum TailtimeBlocks {
    Zero,
    One,
    Two,
}

const OVERSAMPLING_QUALITY: usize = 0;

impl OverSampleType {
    fn value(&self) -> usize {
        match self {
            OverSampleType::None => 1,
            OverSampleType::Double => 2,
            OverSampleType::Quadruple => 4,
        }
    }
}

type WaveShaperCurve = Option<Vec<f32>>;

#[derive(Clone, Debug)]
pub struct WaveShaperNodeOptions {
    pub curve: WaveShaperCurve,
    pub oversample: OverSampleType,
}

impl Default for WaveShaperNodeOptions {
    fn default() -> Self {
        WaveShaperNodeOptions {
            curve: None,
            oversample: OverSampleType::None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum WaveShaperNodeMessage {
    SetCurve(WaveShaperCurve),
}

#[derive(AudioNodeCommon)]
pub(crate) struct WaveShaperNode {
    curve_set: bool,
    curve: WaveShaperCurve,
    #[allow(dead_code)]
    oversample: OverSampleType,
    channel_info: ChannelInfo,
    upsampler: Option<SpeexResamplerState>,
    downsampler: Option<SpeexResamplerState>,
    tailtime_blocks_left: TailtimeBlocks,
}

impl WaveShaperNode {
    pub fn new(options: WaveShaperNodeOptions, channel_info: ChannelInfo) -> Self {
        if let Some(vec) = &options.curve {
            assert!(
                vec.len() > 1,
                "WaveShaperNode curve must have length of 2 or more"
            )
        }

        Self {
            curve_set: options.curve.is_some(),
            curve: options.curve,
            oversample: options.oversample,
            channel_info,
            upsampler: None,
            downsampler: None,
            tailtime_blocks_left: TailtimeBlocks::Zero,
        }
    }

    fn handle_waveshaper_message(&mut self, message: WaveShaperNodeMessage, _sample_rate: f32) {
        match message {
            WaveShaperNodeMessage::SetCurve(new_curve) => {
                if self.curve_set && new_curve.is_some() {
                    panic!("InvalidStateError: cant set curve if it was already set");
                }
                self.curve_set = new_curve.is_some();
                self.curve = new_curve;
            },
        }
    }
}

impl AudioNodeEngine for WaveShaperNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::WaveShaperNode
    }

    fn process(&mut self, mut inputs: Chunk, info: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == 1);

        if self.curve.is_none() {
            return inputs;
        }

        let curve = &self.curve.as_ref().expect("Just checked for is_none()");

        if inputs.blocks[0].is_silence() {
            if WaveShaperNode::silence_produces_nonsilent_output(curve) {
                inputs.blocks[0].explicit_silence();
                self.tailtime_blocks_left = TailtimeBlocks::Two;
            } else if self.tailtime_blocks_left != TailtimeBlocks::Zero {
                inputs.blocks[0].explicit_silence();

                self.tailtime_blocks_left = match self.tailtime_blocks_left {
                    TailtimeBlocks::Zero => TailtimeBlocks::Zero,
                    TailtimeBlocks::One => TailtimeBlocks::Zero,
                    TailtimeBlocks::Two => TailtimeBlocks::One,
                }
            } else {
                return inputs;
            }
        } else {
            self.tailtime_blocks_left = TailtimeBlocks::Two;
        }

        let block = &mut inputs.blocks[0];
        let channels = block.chan_count();

        if self.oversample != OverSampleType::None {
            let rate: usize = info.sample_rate as usize;
            let sampling_factor = self.oversample.value();

            if self.upsampler.is_none() {
                self.upsampler = Some(
                    SpeexResamplerState::new(
                        channels as usize,
                        rate,
                        rate * sampling_factor,
                        OVERSAMPLING_QUALITY,
                    )
                    .expect("Couldnt create upsampler"),
                );
            };

            if self.downsampler.is_none() {
                self.downsampler = Some(
                    SpeexResamplerState::new(
                        channels as usize,
                        rate * sampling_factor,
                        rate,
                        OVERSAMPLING_QUALITY,
                    )
                    .expect("Couldnt create downsampler"),
                );
            };

            let upsampler = self.upsampler.as_mut().unwrap();
            let downsampler = self.downsampler.as_mut().unwrap();

            let mut oversampled_buffer: Vec<f32> =
                vec![0.; FRAMES_PER_BLOCK_USIZE * sampling_factor];

            for chan in 0..channels {
                let out_len = WaveShaperNode::resample(
                    upsampler,
                    chan,
                    block.data_chan(chan),
                    &mut oversampled_buffer,
                );

                debug_assert!(
                    out_len == 128 * sampling_factor,
                    "Expected {} samples in output after upsampling, got: {}",
                    128 * sampling_factor,
                    out_len
                );

                WaveShaperNode::apply_curve(&mut oversampled_buffer, curve);

                let out_len = WaveShaperNode::resample(
                    downsampler,
                    chan,
                    &oversampled_buffer,
                    block.data_chan_mut(chan),
                );

                debug_assert!(
                    out_len == 128,
                    "Expected 128 samples in output after downsampling, got {}",
                    out_len
                );
            }
        } else {
            WaveShaperNode::apply_curve(block.data_mut(), curve);
        }

        inputs
    }

    make_message_handler!(WaveShaperNode: handle_waveshaper_message);
}

impl WaveShaperNode {
    fn silence_produces_nonsilent_output(curve: &[f32]) -> bool {
        let len = curve.len();
        let len_halved = ((len - 1) as f32) / 2.;
        let curve_index: f32 = len_halved;
        let index_lo = curve_index as usize;
        let index_hi = index_lo + 1;
        let interp_factor: f32 = curve_index - index_lo as f32;
        let shaped_val = (1. - interp_factor) * curve[index_lo] + interp_factor * curve[index_hi];
        shaped_val == 0.0
    }

    fn apply_curve(buf: &mut [f32], curve: &[f32]) {
        let len = curve.len();
        let len_halved = ((len - 1) as f32) / 2.;
        buf.iter_mut().for_each(|sample| {
            let curve_index: f32 = len_halved * (*sample + 1.);

            if curve_index <= 0. {
                *sample = curve[0];
            } else if curve_index >= (len - 1) as f32 {
                *sample = curve[len - 1];
            } else {
                let index_lo = curve_index as usize;
                let index_hi = index_lo + 1;
                let interp_factor: f32 = curve_index - index_lo as f32;
                *sample = (1. - interp_factor) * curve[index_lo] + interp_factor * curve[index_hi];
            }
        });
    }

    fn resample(
        st: &mut SpeexResamplerState,
        chan: u8,
        input: &[f32],
        output: &mut [f32],
    ) -> usize {
        let (_in_len, out_len) = st
            .process_float(chan as usize, input, output)
            .expect("Resampling failed");
        out_len
    }
}
