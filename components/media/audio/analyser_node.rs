/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp;
use std::f32::consts::PI;

use crate::block::{Block, Chunk, FRAMES_PER_BLOCK_USIZE};
use crate::node::{AudioNodeEngine, AudioNodeType, BlockInfo, ChannelInfo, ChannelInterpretation};

#[derive(AudioNodeCommon)]
pub(crate) struct AnalyserNode {
    channel_info: ChannelInfo,
    callback: Box<dyn FnMut(Block) + Send>,
}

impl AnalyserNode {
    pub fn new(callback: Box<dyn FnMut(Block) + Send>, channel_info: ChannelInfo) -> Self {
        Self {
            callback,
            channel_info,
        }
    }
}

impl AudioNodeEngine for AnalyserNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::AnalyserNode
    }

    fn process(&mut self, inputs: Chunk, _: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == 1);

        let mut push = inputs.blocks[0].clone();
        push.mix(1, ChannelInterpretation::Speakers);

        (self.callback)(push);

        // analyser node doesn't modify the inputs
        inputs
    }
}

/// From <https://webaudio.github.io/web-audio-api/#dom-analysernode-fftsize>
pub const MAX_FFT_SIZE: usize = 32768;
pub const MAX_BLOCK_COUNT: usize = MAX_FFT_SIZE / FRAMES_PER_BLOCK_USIZE;

/// The actual analysis is done on the DOM side. We provide
/// the actual base functionality in this struct, so the DOM
/// just has to do basic shimming
pub struct AnalysisEngine {
    /// The number of past sample-frames to consider in the FFT
    fft_size: usize,
    smoothing_constant: f64,
    min_decibels: f64,
    max_decibels: f64,
    /// This is a ring buffer containing the last MAX_FFT_SIZE
    /// sample-frames
    data: Box<[f32; MAX_FFT_SIZE]>,
    /// The index of the current block
    current_block: usize,
    /// Have we computed the FFT already?
    fft_computed: bool,
    /// Cached blackman window data
    blackman_windows: Vec<f32>,
    /// The smoothed FFT data (in frequency domain)
    smoothed_fft_data: Vec<f32>,
    /// The computed FFT data, in decibels
    computed_fft_data: Vec<f32>,
    /// The windowed time domain data
    /// Used during FFT computation
    windowed: Vec<f32>,
}

impl AnalysisEngine {
    pub fn new(
        fft_size: usize,
        smoothing_constant: f64,
        min_decibels: f64,
        max_decibels: f64,
    ) -> Self {
        debug_assert!((32..=32768).contains(&fft_size));
        // must be a power of two
        debug_assert!(fft_size & (fft_size - 1) == 0);
        debug_assert!((0. ..=1.).contains(&smoothing_constant));
        debug_assert!(max_decibels > min_decibels);
        Self {
            fft_size,
            smoothing_constant,
            min_decibels,
            max_decibels,
            data: Box::new([0.; MAX_FFT_SIZE]),
            current_block: MAX_BLOCK_COUNT - 1,
            fft_computed: false,
            blackman_windows: Vec::with_capacity(fft_size),
            computed_fft_data: Vec::with_capacity(fft_size / 2),
            smoothed_fft_data: Vec::with_capacity(fft_size / 2),
            windowed: Vec::with_capacity(fft_size),
        }
    }

    pub fn set_fft_size(&mut self, fft_size: usize) {
        debug_assert!((32..=32768).contains(&fft_size));
        // must be a power of two
        debug_assert!(fft_size & (fft_size - 1) == 0);
        self.fft_size = fft_size;
        self.fft_computed = false;
    }

    pub fn get_fft_size(&self) -> usize {
        self.fft_size
    }

    pub fn set_smoothing_constant(&mut self, smoothing_constant: f64) {
        debug_assert!((0. ..=1.).contains(&smoothing_constant));
        self.smoothing_constant = smoothing_constant;
        self.fft_computed = false;
    }

    pub fn get_smoothing_constant(&self) -> f64 {
        self.smoothing_constant
    }

    pub fn set_min_decibels(&mut self, min_decibels: f64) {
        debug_assert!(min_decibels < self.max_decibels);
        self.min_decibels = min_decibels;
    }

    pub fn get_min_decibels(&self) -> f64 {
        self.min_decibels
    }

    pub fn set_max_decibels(&mut self, max_decibels: f64) {
        debug_assert!(self.min_decibels < max_decibels);
        self.max_decibels = max_decibels;
    }

    pub fn get_max_decibels(&self) -> f64 {
        self.max_decibels
    }

    fn advance(&mut self) {
        self.current_block += 1;
        if self.current_block >= MAX_BLOCK_COUNT {
            self.current_block = 0;
        }
    }

    /// Get the data of the current block
    fn curent_block_mut(&mut self) -> &mut [f32] {
        let index = FRAMES_PER_BLOCK_USIZE * self.current_block;
        &mut self.data[index..(index + FRAMES_PER_BLOCK_USIZE)]
    }

    /// Given an index from 0 to fft_size, convert it into an index into
    /// the backing array
    fn convert_index(&self, index: usize) -> usize {
        let offset = self.fft_size - index;
        let last_element = (1 + self.current_block) * FRAMES_PER_BLOCK_USIZE - 1;
        if offset > last_element {
            MAX_FFT_SIZE - offset + last_element
        } else {
            last_element - offset
        }
    }

    /// Given an index into the backing array, increment it
    fn advance_index(&self, index: &mut usize) {
        *index += 1;
        if *index >= MAX_FFT_SIZE {
            *index = 0;
        }
    }

    pub fn push(&mut self, mut block: Block) {
        debug_assert!(block.chan_count() == 1);
        self.advance();
        if !block.is_silence() {
            self.curent_block_mut().copy_from_slice(block.data_mut());
        }
        self.fft_computed = false;
    }

    /// <https://webaudio.github.io/web-audio-api/#blackman-window>
    fn compute_blackman_windows(&mut self) {
        if self.blackman_windows.len() == self.fft_size {
            return;
        }
        const ALPHA: f32 = 0.16;
        const ALPHA_0: f32 = (1. - ALPHA) / 2.;
        const ALPHA_1: f32 = 1. / 2.;
        const ALPHA_2: f32 = ALPHA / 2.;
        self.blackman_windows.resize(self.fft_size, 0.);
        let coeff = PI * 2. / self.fft_size as f32;
        for n in 0..self.fft_size {
            self.blackman_windows[n] = ALPHA_0 - ALPHA_1 * (coeff * n as f32).cos() +
                ALPHA_2 * (2. * coeff * n as f32).cos();
        }
    }

    fn apply_blackman_window(&mut self) {
        self.compute_blackman_windows();
        self.windowed.resize(self.fft_size, 0.);

        let mut data_idx = self.convert_index(0);
        for n in 0..self.fft_size {
            self.windowed[n] = self.blackman_windows[n] * self.data[data_idx];
            self.advance_index(&mut data_idx);
        }
    }

    fn compute_fft(&mut self) {
        if self.fft_computed {
            return;
        }
        self.fft_computed = true;
        self.apply_blackman_window();
        self.computed_fft_data.resize(self.fft_size / 2, 0.);
        self.smoothed_fft_data.resize(self.fft_size / 2, 0.);

        for k in 0..(self.fft_size / 2) {
            let mut sum_real = 0.;
            let mut sum_imaginary = 0.;
            let factor = -2. * PI * k as f32 / self.fft_size as f32;
            for n in 0..(self.fft_size) {
                sum_real += self.windowed[n] * (factor * n as f32).cos();
                sum_imaginary += self.windowed[n] * (factor * n as f32).sin();
            }
            let sum_real = sum_real / self.fft_size as f32;
            let sum_imaginary = sum_imaginary / self.fft_size as f32;
            let magnitude = (sum_real * sum_real + sum_imaginary * sum_imaginary).sqrt();
            self.smoothed_fft_data[k] = (self.smoothing_constant * self.smoothed_fft_data[k] as f64 +
                (1. - self.smoothing_constant) * magnitude as f64)
                as f32;
            self.computed_fft_data[k] = 20. * self.smoothed_fft_data[k].log(10.);
        }
    }

    pub fn fill_time_domain_data(&self, dest: &mut [f32]) {
        let mut data_idx = self.convert_index(0);
        let end = cmp::min(self.fft_size, dest.len());
        for entry in &mut dest[0..end] {
            *entry = self.data[data_idx];
            self.advance_index(&mut data_idx);
        }
    }

    pub fn fill_byte_time_domain_data(&self, dest: &mut [u8]) {
        let mut data_idx = self.convert_index(0);
        let end = cmp::min(self.fft_size, dest.len());
        for entry in &mut dest[0..end] {
            let result = 128. * (1. + self.data[data_idx]);
            *entry = clamp_255(result);
            self.advance_index(&mut data_idx)
        }
    }

    pub fn fill_frequency_data(&mut self, dest: &mut [f32]) {
        self.compute_fft();
        let len = cmp::min(dest.len(), self.computed_fft_data.len());
        dest[0..len].copy_from_slice(&self.computed_fft_data[0..len]);
    }

    pub fn fill_byte_frequency_data(&mut self, dest: &mut [u8]) {
        self.compute_fft();
        let len = cmp::min(dest.len(), self.computed_fft_data.len());
        let ratio = 255. / (self.max_decibels - self.min_decibels);
        for (index, freq) in dest[0..len].iter_mut().enumerate() {
            let result = ratio * (self.computed_fft_data[index] as f64 - self.min_decibels);
            *freq = clamp_255(result as f32);
        }
    }
}

fn clamp_255(val: f32) -> u8 {
    if val > 255. {
        255
    } else if val < 0. {
        0
    } else {
        val as u8
    }
}
