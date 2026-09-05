/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::f32::consts::PI as PI_32;
use std::f64::consts::PI;
use std::iter::zip;

use log::error;
use malloc_size_of_derive::MallocSizeOf;
use num_complex::Complex32;
use num_traits::Zero;
use realfft::RealFftPlanner;

use crate::oscillator_node::OscillatorType;

// https://webaudio.github.io/web-audio-api/#PeriodicWave
// A conforming implementation must support up to at least 8192 elements
// The wavetable design is based on Blink's implementation
// https://github.com/mozilla-firefox/firefox/blob/main/dom/media/webaudio/blink/PeriodicWave.cpp
const FFT_MAX_SIZE: usize = 8192;
const FFT_MIN_SIZE: usize = 4096;
const CENTS_PER_RANGE: u32 = 1200 / 3; // Represents 1/3 of an octave

#[derive(Clone, Debug, Default, MallocSizeOf)]
pub struct PeriodicWaveOptions {
    imag: Vec<f32>,
    real: Vec<f32>,
    disable_normalization: bool,
}

impl PeriodicWaveOptions {
    pub fn new(imag: Vec<f32>, real: Vec<f32>, disable_normalization: bool) -> Self {
        Self {
            imag,
            real,
            disable_normalization,
        }
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
pub struct PeriodicWave {
    pub imag: Vec<f32>,
    pub real: Vec<f32>,
    pub normalize: bool,
    size: usize,
    wavetable: Wavetable,
}

type Wave = Vec<f32>;

#[derive(Clone, Debug, MallocSizeOf)]
struct Wavetable {
    max_number_of_partials_in_band_limited_table: usize,
    normalization_factor: Option<f64>,
    waves: Vec<Wave>,
    size: usize,
}

impl Wavetable {
    fn new(size: usize, number_of_waves: usize) -> Self {
        Wavetable {
            max_number_of_partials_in_band_limited_table: 0,
            normalization_factor: None,
            waves: vec![Vec::with_capacity(size); number_of_waves],
            size,
        }
    }

    fn max_number_of_partials(&self) -> usize {
        self.size / 2
    }

    // The range index maps a frequency to a bucket that represents 1/3 of an octave
    fn number_of_partials_for_range_index(&self, index: usize) -> usize {
        // Number of cents below Nyquist where we drop the partials
        let cents_threshold = index as f64 * CENTS_PER_RANGE as f64;

        // Represents the fraction of partials we keep. Essentially, each octave higher halves the
        // number of partials
        let fraction_to_keep = (-cents_threshold / 1200.0).exp2();
        (fraction_to_keep * self.max_number_of_partials() as f64) as usize
    }

    // Approximate the band limited wave by calculating an inverse FFT
    fn calculate_unnormalized_wave(
        &self,
        real: &[f32],
        imag: &[f32],
        frequency: f64,
        sample_rate: f64,
        num_partials: usize,
    ) -> Wave {
        let coefficients_length = real.len();
        let mut num_partials = num_partials.min(coefficients_length);

        // Limit number of partials to those below Nyquist frequency
        let nyquist = 0.5 * sample_rate;
        if !frequency.is_zero() {
            num_partials = num_partials.min((nyquist / frequency) as usize);
        }

        // The length of real output needs to be the size of the periodic wave, N
        // Complex input must be length N/2 + 1. The max number of partials we take from the given
        // Fourier coefficients is constructed to be N/2
        let input_length = self.size / 2 + 1;
        let mut complex_input = vec![Complex32::ZERO; input_length];

        for (i, (a, b)) in zip(real, imag).take(num_partials).enumerate() {
            complex_input[i] = Complex32::new(*a, -(*b));
        }

        let mut planner = RealFftPlanner::new();
        let complex_to_real_fft = planner.plan_fft_inverse(self.size);
        let mut real_output = complex_to_real_fft.make_output_vec();
        if complex_to_real_fft
            .process(&mut complex_input, &mut real_output)
            .is_ok()
        {
            real_output
        } else {
            // This should not happen given how input_length has been calculated
            error!(
                "Error calculating IFFT from periodic wave coefficients. By construction lengths of complex input and real output should be N/2 + 1 and N respectively"
            );
            Vec::with_capacity(self.size)
        }
    }

    fn insert_wave(
        &mut self,
        real: &[f32],
        imag: &[f32],
        frequency: f64,
        sample_rate: f64,
        normalize: bool,
        range_index: usize,
    ) {
        let mut wave = self.calculate_unnormalized_wave(
            real,
            imag,
            frequency,
            sample_rate,
            self.number_of_partials_for_range_index(range_index),
        );
        let normalization_factor = if normalize {
            // Calculate the normalization factor if necessary
            self.normalization_factor.unwrap_or_else(|| {
                wave.iter()
                    .map(|x| (*x as f64).abs())
                    .fold(0.0, |acc, f| f.max(acc))
            })
        } else {
            // If normalize is false, we still need to divide the IFFT output by 2.
            // The reason is that realfft takes the complex input as the positive frequencies and
            // then constructs the negative frequencies in a symmetric manner (reverse order of
            // complex conjugate of the input).
            2.0
        };
        if !normalization_factor.is_zero() {
            wave = wave
                .iter()
                .map(|x| ((*x as f64) / normalization_factor) as f32)
                .collect();
        }
        self.waves[range_index] = wave;
    }

    fn get_wave(&self, range_index: usize) -> Option<&Wave> {
        self.waves.get(range_index)
    }

    fn has_wave(&self, range_index: usize) -> bool {
        self.waves
            .get(range_index)
            .is_some_and(|wave| !wave.is_empty())
    }
}

impl PeriodicWave {
    pub fn new(options: PeriodicWaveOptions) -> Self {
        let number_of_components = options.imag.len();
        let size = if number_of_components <= FFT_MIN_SIZE {
            FFT_MIN_SIZE
        } else {
            number_of_components.next_power_of_two().min(FFT_MAX_SIZE)
        };
        // 3 tables per octave
        let number_of_waves = (3 * size.ilog2()) as usize;
        let wavetable = Wavetable::new(size, number_of_waves);
        Self {
            imag: options.imag,
            real: options.real,
            normalize: !options.disable_normalization,
            size,
            wavetable,
        }
    }

    fn get_or_insert_wave(&mut self, frequency: f64, sample_rate: f64) -> (&Wave, &Wave, f64) {
        // Frequencies can be negative, so alias to the positive frequency
        let frequency = frequency.abs();
        // If frequency is low enough such that we can take more partials from the given Fourier
        // coefficients, reconstruct the wavetable
        let mut num_partials = self.wavetable.number_of_partials_for_range_index(0);
        let nyquist = 0.5 * sample_rate;
        if !frequency.is_zero() {
            num_partials = num_partials.min((nyquist / frequency) as usize);
        }
        if num_partials > self.wavetable.max_number_of_partials_in_band_limited_table {
            let number_of_waves = self.wavetable.waves.capacity();
            let mut wavetable = Wavetable::new(self.size, number_of_waves);
            // Create the first table in order to get the new normalization factor. The first table
            // is constructed such that it has the most partials, as it maps to the lowest frequencies
            wavetable.insert_wave(
                &self.real,
                &self.imag,
                frequency,
                sample_rate,
                self.normalize,
                0,
            );
            wavetable.max_number_of_partials_in_band_limited_table = num_partials;
            self.wavetable = wavetable;
        }

        let number_of_waves = self.wavetable.waves.capacity();

        // Calculate the pitch range.
        let min_fundamental_frequency = sample_rate / (self.size as f64);
        let ratio = if frequency.is_zero() {
            0.5
        } else {
            frequency / min_fundamental_frequency
        };
        let cents_above_lowest_frequency = ratio.log2() * 1200.0;

        // Round up to the next range to truncate partials before aliasing occurs
        let mut pitch_range = 1.0 + cents_above_lowest_frequency / CENTS_PER_RANGE as f64;
        pitch_range = pitch_range.max(0.0);
        pitch_range = pitch_range.min(self.wavetable.waves.capacity() as f64 - 1.0);

        // The words "lower" and "higher" refer to the wave data having the lower and higher numbers of partials.
        // As the range index gets larger, the more partials we cull out.
        // The "lower" table data will have a larger range index.
        // Conceptually, higher frequencies will lookup higher range index,
        // which have less partials due to higher orders exceeding Nyquist.
        let higher_data_range_index = pitch_range as usize;
        let lower_data_range_index = if higher_data_range_index < number_of_waves - 1 {
            higher_data_range_index + 1
        } else {
            higher_data_range_index
        };

        // Check if the wavetable has a wave for given range index. If not, create it.
        if !self.wavetable.has_wave(lower_data_range_index) {
            self.wavetable.insert_wave(
                &self.real,
                &self.imag,
                frequency,
                sample_rate,
                self.normalize,
                lower_data_range_index,
            );
        }
        if !self.wavetable.has_wave(higher_data_range_index) {
            self.wavetable.insert_wave(
                &self.real,
                &self.imag,
                frequency,
                sample_rate,
                self.normalize,
                higher_data_range_index,
            );
        }
        let lower_wave_data = self
            .wavetable
            .get_wave(lower_data_range_index)
            .expect("Range index is calculated to be within bounds");
        let higher_wave_data = self
            .wavetable
            .get_wave(higher_data_range_index)
            .expect("Range index is calculated to be within bounds");

        // Ranges from 0 -> 1 to interpolate between lower -> higher.
        let table_interpolation_factor = lower_data_range_index as f64 - pitch_range;
        (
            lower_wave_data,
            higher_wave_data,
            table_interpolation_factor,
        )
    }

    pub(crate) fn calculate_waveform(
        &mut self,
        frequency: f64,
        sample_rate: f64,
        phase: f64,
    ) -> f64 {
        // Convert the phase which is in radians [0, 2PI), to the position of the
        // approximated wave which ranges from [0, N)
        let position = phase / (2.0 * PI) * self.size as f64;
        let position_floor = position.floor();
        let position_interpolation_factor = position - position_floor;
        let index_mask = self.size - 1;
        let (lower_wave_data, higher_wave_data, table_interpolation_factor) =
            self.get_or_insert_wave(frequency, sample_rate);
        let position_index = position_floor as usize;
        // Use an index mask to handle the wrap around case where position = self.size. This is more
        // efficient than checking the position index, less branching
        let lower_position_index = position_index & index_mask;
        let higher_position_index = (lower_position_index + 1) & index_mask;
        // Linear interpolation of the higher and lower position index
        // to calculate the lower and higher wave values
        let lower = (1.0 - position_interpolation_factor) *
            lower_wave_data[lower_position_index] as f64 +
            position_interpolation_factor * lower_wave_data[higher_position_index] as f64;
        let higher = (1.0 - position_interpolation_factor) *
            higher_wave_data[lower_position_index] as f64 +
            position_interpolation_factor * higher_wave_data[higher_position_index] as f64;
        // Linear interpolation of the lower and higher waves
        (1.0 - table_interpolation_factor) * lower + table_interpolation_factor * higher
    }

    pub(crate) fn generate_waveform_coefficients(oscillator_type: OscillatorType) -> PeriodicWave {
        let mut periodic_wave = PeriodicWave::new(PeriodicWaveOptions::new(vec![], vec![], false));
        periodic_wave.real = vec![0.0; periodic_wave.size / 2 + 1];
        periodic_wave.imag = vec![0.0; periodic_wave.size / 2 + 1];
        match oscillator_type {
            // Custom will default to Sine in this case
            OscillatorType::Sine => {
                periodic_wave.imag[1] = 1.0;
            },
            OscillatorType::Square => {
                for n in 1..periodic_wave.imag.len() {
                    periodic_wave.imag[n] =
                        (2.0 / (n as f32 * PI_32)) * (1.0 - (-1.0_f32).powi(n as i32));
                }
            },
            OscillatorType::Sawtooth => {
                for n in 1..periodic_wave.imag.len() {
                    periodic_wave.imag[n] =
                        (-1.0_f32).powi(n as i32 + 1) * 2.0 / (n as f32 * PI_32);
                }
            },
            OscillatorType::Triangle => {
                for n in 1..periodic_wave.imag.len() {
                    periodic_wave.imag[n] =
                        (8.0 * (n as f32 * PI_32 / 2.0).sin()) / (PI_32 * n as f32).powi(2);
                }
            },
            OscillatorType::Custom(w) => {
                return w;
            },
        }
        periodic_wave
    }
}
