/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::f32::consts::SQRT_2;
use std::mem;
use std::ops::*;

use byte_slice_cast::*;
use euclid::default::Vector3D;
use smallvec::{SmallVec, smallvec};

use crate::graph::{PortIndex, PortKind};
use crate::node::ChannelInterpretation;

// defined by spec
// https://webaudio.github.io/web-audio-api/#render-quantum
pub const FRAMES_PER_BLOCK: Tick = Tick(128);
pub const FRAMES_PER_BLOCK_USIZE: usize = FRAMES_PER_BLOCK.0 as usize;

/// A tick, i.e. the time taken for a single frame
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Tick(pub u64);

/// A collection of blocks received as input by a node
/// or outputted by a node.
///
/// This will usually be a single block.
///
/// Some nodes have multiple inputs or outputs, which is
/// where this becomes useful. Source nodes have an input
/// of an empty chunk.
pub struct Chunk {
    pub blocks: SmallVec<[Block; 1]>,
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            blocks: SmallVec::new(),
        }
    }
}

impl Chunk {
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn explicit_silence() -> Self {
        let mut block = Block::default();
        block.explicit_silence();
        let blocks = smallvec![block];
        Self { blocks }
    }
}

/// We render audio in blocks of size FRAMES_PER_BLOCK
///
/// A single block may contain multiple channels
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    /// The number of channels in this block
    channels: u8,
    /// This is an optimization which means that the buffer is representing multiple channels with the
    /// same content at once. Happens when audio is upmixed or when a source like
    /// an oscillator node has multiple channel outputs
    repeat: bool,
    /// If this vector is empty, it is a shorthand for "silence"
    /// It is possible to obtain an explicitly silent buffer via .explicit_silence()
    ///
    /// This must be of length channels * FRAMES_PER_BLOCK, unless `repeat` is true,
    /// in which case it will be of length FRAMES_PER_BLOCK
    buffer: Vec<f32>,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            channels: 1,
            repeat: false,
            buffer: Vec::new(),
        }
    }
}

impl Block {
    /// Empty block with no channels, for pushing
    /// new channels to.
    ///
    /// Must be used with push_chan
    pub fn empty() -> Self {
        Block {
            channels: 0,
            ..Default::default()
        }
    }

    pub fn for_channels_explicit(channels: u8) -> Self {
        Block {
            channels,
            repeat: false,
            buffer: vec![0.; FRAMES_PER_BLOCK_USIZE * channels as usize],
        }
    }

    /// This provides the entire buffer as a mutable slice of u8
    pub fn as_mut_byte_slice(&mut self) -> &mut [u8] {
        self.data_mut().as_mut_byte_slice()
    }

    pub fn for_vec(buffer: Vec<f32>) -> Self {
        assert!(buffer.len() % FRAMES_PER_BLOCK_USIZE == 0);
        Block {
            channels: (buffer.len() / FRAMES_PER_BLOCK_USIZE) as u8,
            repeat: false,
            buffer,
        }
    }

    /// Zero-gain sum with another buffer
    ///
    /// Used after mixing multiple inputs to a single port
    pub fn sum(mut self, mut other: Self) -> Self {
        if self.is_silence() {
            other
        } else if other.is_silence() {
            self
        } else {
            debug_assert_eq!(self.channels, other.channels);
            if self.repeat ^ other.repeat {
                self.explicit_repeat();
                other.explicit_repeat();
            }
            debug_assert_eq!(self.buffer.len(), other.buffer.len());
            for (a, b) in self.buffer.iter_mut().zip(other.buffer.iter()) {
                *a += b
            }
            self
        }
    }

    /// If this is in "silence" mode without a buffer, allocate a silent buffer
    pub fn explicit_silence(&mut self) {
        if self.buffer.is_empty() {
            self.buffer.resize(FRAMES_PER_BLOCK_USIZE, 0.);
            self.repeat = true;
        }
    }

    /// This provides the entire buffer as a mutable slice of f32
    pub fn data_mut(&mut self) -> &mut [f32] {
        self.explicit_silence();
        &mut self.buffer
    }

    pub fn explicit_repeat(&mut self) {
        if self.repeat {
            debug_assert!(self.buffer.len() == FRAMES_PER_BLOCK_USIZE);
            if self.channels > 1 {
                let mut new = Vec::with_capacity(FRAMES_PER_BLOCK_USIZE * self.channels as usize);
                for _ in 0..self.channels {
                    new.extend(&self.buffer)
                }

                self.buffer = new;
            }
            self.repeat = false;
        } else if self.is_silence() {
            self.buffer
                .resize(FRAMES_PER_BLOCK_USIZE * self.channels as usize, 0.);
        }
    }

    pub fn data_chan_mut(&mut self, chan: u8) -> &mut [f32] {
        self.explicit_repeat();
        let start = chan as usize * FRAMES_PER_BLOCK_USIZE;
        &mut self.buffer[start..start + FRAMES_PER_BLOCK_USIZE]
    }

    #[inline]
    pub fn data_chan(&self, chan: u8) -> &[f32] {
        debug_assert!(
            !self.is_silence(),
            "data_chan doesn't work with silent buffers"
        );
        let offset = if self.repeat {
            0
        } else {
            chan as usize * FRAMES_PER_BLOCK_USIZE
        };
        &self.buffer[offset..offset + FRAMES_PER_BLOCK_USIZE]
    }

    pub fn take(&mut self) -> Block {
        let new = Block {
            channels: self.channels,
            ..Default::default()
        };
        mem::replace(self, new)
    }

    pub fn chan_count(&self) -> u8 {
        self.channels
    }

    pub fn iter(&mut self) -> FrameIterator<'_> {
        FrameIterator::new(self)
    }

    pub fn is_silence(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn is_repeat(&self) -> bool {
        self.repeat
    }

    pub fn data_chan_frame(&self, frame: usize, chan: u8) -> f32 {
        if self.is_silence() {
            0.
        } else {
            self.data_chan(chan)[frame]
        }
    }

    pub fn push_chan(&mut self, data: &[f32]) {
        assert!(!self.repeat);
        assert!(!self.is_silence() || self.channels == 0);
        assert!(data.len() == FRAMES_PER_BLOCK_USIZE);
        self.buffer.extend(data);
        self.channels += 1;
    }

    /// upmix/downmix the channels if necessary
    ///
    /// Currently only supports upmixing from 1
    pub fn mix(&mut self, channels: u8, interpretation: ChannelInterpretation) {
        // If we're not changing the number of channels, we
        // don't actually need to mix
        if self.channels == channels {
            return;
        }

        // Silent buffers stay silent
        if self.is_silence() {
            self.channels = channels;
            return;
        }

        if interpretation == ChannelInterpretation::Discrete {
            // discrete downmixes by truncation, upmixes by adding
            // silent channels

            // If we're discrete, have a repeat, and are downmixing,
            // just truncate by changing the channel value
            if self.repeat && self.channels > channels {
                self.channels = channels;
            } else {
                // otherwise resize the buffer, silent-filling when necessary
                self.resize_silence(channels);
            }
        } else {
            // For speakers, we have to do special things based on the
            // interpretation of the channels for each kind of speakers

            // The layout of each speaker kind is:
            //
            // - Mono: [The mono channel]
            // - Stereo: [L, R]
            // - Quad: [L, R, SL, SR]
            // - 5.1: [L, R, C, LFE, SL, SR]

            match (self.channels, channels) {
                // Upmixing
                // https://webaudio.github.io/web-audio-api/#UpMix-sub

                // mono
                (1, 2) => {
                    // output.{L, R} = input
                    self.repeat(2);
                },
                (1, 4) => {
                    // output.{L, R} = input
                    self.repeat(2);
                    // output.{SL, SR} = 0
                    self.resize_silence(4);
                },
                (1, 6) => {
                    let mut v = Vec::with_capacity(channels as usize * FRAMES_PER_BLOCK_USIZE);
                    // output.{L, R} = 0
                    v.resize(2 * FRAMES_PER_BLOCK_USIZE, 0.);
                    // output.C = input
                    v.extend(&self.buffer);
                    self.buffer = v;
                    // output.{LFE, SL, SR} = 0
                    self.resize_silence(6);
                },

                // stereo
                (2, 4) | (2, 6) => {
                    // output.{L, R} = input.{L, R}
                    // (5.1) output.{C, LFE} = 0
                    // output.{SL, SR} = 0
                    self.resize_silence(channels);
                },

                // quad
                (4, 6) => {
                    // we can avoid this and instead calculate offsets
                    // based off whether or not this is `repeat`, but
                    // a `repeat` quad block should be rare
                    self.explicit_repeat();

                    let mut v = Vec::with_capacity(6 * FRAMES_PER_BLOCK_USIZE);
                    // output.{L, R} = input.{L, R}
                    v.extend(&self.buffer[0..2 * FRAMES_PER_BLOCK_USIZE]);
                    // output.{C, LFE} = 0
                    v.resize(4 * FRAMES_PER_BLOCK_USIZE, 0.);
                    // output.{SL, R} = input.{SL, SR}
                    v.extend(&self.buffer[2 * FRAMES_PER_BLOCK_USIZE..]);
                    self.buffer = v;
                    self.channels = channels;
                },

                // Downmixing
                // https://webaudio.github.io/web-audio-api/#down-mix

                // mono
                (2, 1) => {
                    let mut v = Vec::with_capacity(FRAMES_PER_BLOCK_USIZE);
                    for frame in 0..FRAMES_PER_BLOCK_USIZE {
                        // output = 0.5 * (input.L + input.R);
                        let o =
                            0.5 * (self.data_chan_frame(frame, 0) + self.data_chan_frame(frame, 1));
                        v.push(o);
                    }
                    self.buffer = v;
                    self.channels = 1;
                    self.repeat = false;
                },
                (4, 1) => {
                    let mut v = Vec::with_capacity(FRAMES_PER_BLOCK_USIZE);
                    for frame in 0..FRAMES_PER_BLOCK_USIZE {
                        // output = 0.5 * (input.L + input.R + input.SL + input.SR);
                        let o = 0.25 *
                            (self.data_chan_frame(frame, 0) +
                                self.data_chan_frame(frame, 1) +
                                self.data_chan_frame(frame, 2) +
                                self.data_chan_frame(frame, 3));
                        v.push(o);
                    }
                    self.buffer = v;
                    self.channels = 1;
                    self.repeat = false;
                },
                (6, 1) => {
                    let mut v = Vec::with_capacity(FRAMES_PER_BLOCK_USIZE);
                    for frame in 0..FRAMES_PER_BLOCK_USIZE {
                        // output = sqrt(0.5) * (input.L + input.R) + input.C + 0.5 * (input.SL + input.SR)
                        let o =
                            // sqrt(0.5) * (input.L + input.R)
                            SQRT_2 * (self.data_chan_frame(frame, 0) +
                                      self.data_chan_frame(frame, 1)) +
                            // input.C
                            self.data_chan_frame(frame, 2) +
                            // (ignore LFE)
                            // + 0 * self.buffer[frame + 3 * FRAMES_PER_BLOCK_USIZE]
                            // 0.5 * (input.SL + input.SR)
                            0.5 * (self.data_chan_frame(frame, 4) +
                                   self.data_chan_frame(frame, 5));
                        v.push(o);
                    }
                    self.buffer = v;
                    self.channels = 1;
                    self.repeat = false;
                },

                // stereo
                (4, 2) => {
                    let mut v = Vec::with_capacity(2 * FRAMES_PER_BLOCK_USIZE);
                    v.resize(2 * FRAMES_PER_BLOCK_USIZE, 0.);
                    for frame in 0..FRAMES_PER_BLOCK_USIZE {
                        // output.L = 0.5 * (input.L + input.SL)
                        v[frame] =
                            0.5 * (self.data_chan_frame(frame, 0) + self.data_chan_frame(frame, 2));
                        // output.R = 0.5 * (input.R + input.SR)
                        v[frame + FRAMES_PER_BLOCK_USIZE] =
                            0.5 * (self.data_chan_frame(frame, 1) + self.data_chan_frame(frame, 3));
                    }
                    self.buffer = v;
                    self.channels = 2;
                    self.repeat = false;
                },
                (6, 2) => {
                    let mut v = Vec::with_capacity(2 * FRAMES_PER_BLOCK_USIZE);
                    v.resize(2 * FRAMES_PER_BLOCK_USIZE, 0.);
                    for frame in 0..FRAMES_PER_BLOCK_USIZE {
                        // output.L = L + sqrt(0.5) * (input.C + input.SL)
                        v[frame] = self.data_chan_frame(frame, 0) +
                            SQRT_2 *
                                (self.data_chan_frame(frame, 2) +
                                    self.data_chan_frame(frame, 4));
                        // output.R = R + sqrt(0.5) * (input.C + input.SR)
                        v[frame + FRAMES_PER_BLOCK_USIZE] = self.data_chan_frame(frame, 1) +
                            SQRT_2 *
                                (self.data_chan_frame(frame, 2) +
                                    self.data_chan_frame(frame, 5));
                    }
                    self.buffer = v;
                    self.channels = 2;
                    self.repeat = false;
                },

                // quad
                (6, 4) => {
                    let mut v = Vec::with_capacity(6 * FRAMES_PER_BLOCK_USIZE);
                    v.resize(6 * FRAMES_PER_BLOCK_USIZE, 0.);
                    for frame in 0..FRAMES_PER_BLOCK_USIZE {
                        // output.L = L + sqrt(0.5) * input.C
                        v[frame] = self.data_chan_frame(frame, 0) +
                            SQRT_2 * self.data_chan_frame(frame, 2);
                        // output.R = R + sqrt(0.5) * input.C
                        v[frame + FRAMES_PER_BLOCK_USIZE] = self.data_chan_frame(frame, 1) +
                            SQRT_2 * self.data_chan_frame(frame, 2);
                        // output.SL = input.SL
                        v[frame + 2 * FRAMES_PER_BLOCK_USIZE] = self.data_chan_frame(frame, 4);
                        // output.SR = input.SR
                        v[frame + 3 * FRAMES_PER_BLOCK_USIZE] = self.data_chan_frame(frame, 5);
                    }
                    self.buffer = v;
                    self.channels = 4;
                    self.repeat = false;
                },

                // If it's not a known kind of speaker configuration, treat as
                // discrete
                _ => {
                    self.mix(channels, ChannelInterpretation::Discrete);
                },
            }
            debug_assert!(self.channels == channels);
        }
    }

    /// Resize to add or remove channels, fill extra channels with silence
    pub fn resize_silence(&mut self, channels: u8) {
        self.explicit_repeat();
        self.buffer
            .resize(FRAMES_PER_BLOCK_USIZE * channels as usize, 0.);
        self.channels = channels;
    }

    /// Take a single-channel block and repeat the
    /// channel
    pub fn repeat(&mut self, channels: u8) {
        debug_assert!(self.channels == 1);
        self.channels = channels;
        if !self.is_silence() {
            self.repeat = true;
        }
    }

    pub fn interleave(&mut self) -> Vec<f32> {
        self.explicit_repeat();
        let mut vec = Vec::with_capacity(self.buffer.len());
        // FIXME this isn't too efficient
        vec.resize(self.buffer.len(), 0.);
        for frame in 0..FRAMES_PER_BLOCK_USIZE {
            let channels = self.channels as usize;
            for chan in 0..channels {
                vec[frame * channels + chan] = self.buffer[chan * FRAMES_PER_BLOCK_USIZE + frame]
            }
        }
        vec
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get the position, forward, and up vectors for a given
    /// AudioListener-produced block
    pub fn listener_data(&self, frame: Tick) -> (Vector3D<f32>, Vector3D<f32>, Vector3D<f32>) {
        let frame = frame.0 as usize;
        (
            Vector3D::new(
                self.data_chan_frame(frame, 0),
                self.data_chan_frame(frame, 1),
                self.data_chan_frame(frame, 2),
            ),
            Vector3D::new(
                self.data_chan_frame(frame, 3),
                self.data_chan_frame(frame, 4),
                self.data_chan_frame(frame, 5),
            ),
            Vector3D::new(
                self.data_chan_frame(frame, 6),
                self.data_chan_frame(frame, 7),
                self.data_chan_frame(frame, 8),
            ),
        )
    }
}

/// An iterator over frames in a block
pub struct FrameIterator<'a> {
    frame: Tick,
    block: &'a mut Block,
}

impl<'a> FrameIterator<'a> {
    #[inline]
    pub fn new(block: &'a mut Block) -> Self {
        FrameIterator {
            frame: Tick(0),
            block,
        }
    }

    /// Advance the iterator
    ///
    /// We can't implement Iterator since it doesn't support
    /// streaming iterators, but we can call `while let Some(frame) = iter.next()`
    /// here
    #[inline]
    pub fn next<'b>(&'b mut self) -> Option<FrameRef<'b>> {
        let curr = self.frame;
        if curr < FRAMES_PER_BLOCK {
            self.frame.advance();
            Some(FrameRef {
                frame: curr,
                block: self.block,
            })
        } else {
            None
        }
    }
}

/// A reference to a frame
pub struct FrameRef<'a> {
    frame: Tick,
    block: &'a mut Block,
}

impl<'a> FrameRef<'a> {
    #[inline]
    pub fn tick(&self) -> Tick {
        self.frame
    }

    /// Given a block and a function `f`, mutate the frame through all channels with `f`
    ///
    /// Use this when you plan to do the same operation for each channel.
    /// (Helpers for the other cases will eventually exist)
    ///
    /// Block must not be silence
    ///
    /// The second parameter to f is the channel number, 0 in case of a repeat()
    #[inline]
    pub fn mutate_with<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut f32, u8),
    {
        debug_assert!(
            !self.block.is_silence(),
            "mutate_frame_with should not be called with a silenced block, \
             call .explicit_silence() if you wish to use this"
        );
        if self.block.repeat {
            f(&mut self.block.buffer[self.frame.0 as usize], 0)
        } else {
            for chan in 0..self.block.channels {
                f(
                    &mut self.block.buffer
                        [chan as usize * FRAMES_PER_BLOCK_USIZE + self.frame.0 as usize],
                    chan,
                )
            }
        }
    }
}

// operator impls

impl<T: PortKind> IndexMut<PortIndex<T>> for Chunk {
    fn index_mut(&mut self, i: PortIndex<T>) -> &mut Block {
        if let PortIndex::Port(i) = i {
            &mut self.blocks[i as usize]
        } else {
            panic!("attempted to index chunk with param")
        }
    }
}

impl<T: PortKind> Index<PortIndex<T>> for Chunk {
    type Output = Block;
    fn index(&self, i: PortIndex<T>) -> &Block {
        if let PortIndex::Port(i) = i {
            &self.blocks[i as usize]
        } else {
            panic!("attempted to index chunk with param")
        }
    }
}

impl Add<Tick> for Tick {
    type Output = Tick;
    fn add(self, other: Tick) -> Self {
        self + other.0
    }
}

impl AddAssign for Tick {
    fn add_assign(&mut self, other: Tick) {
        *self = *self + other
    }
}

impl Sub<Tick> for Tick {
    type Output = Tick;
    fn sub(self, other: Tick) -> Self {
        self - other.0
    }
}

impl Add<u64> for Tick {
    type Output = Tick;
    fn add(self, other: u64) -> Self {
        Tick(self.0 + other)
    }
}

impl Sub<u64> for Tick {
    type Output = Tick;
    fn sub(self, other: u64) -> Self {
        Tick(self.0 - other)
    }
}

impl Div<f64> for Tick {
    type Output = f64;
    fn div(self, other: f64) -> f64 {
        self.0 as f64 / other
    }
}

impl Tick {
    pub const FRAMES_PER_BLOCK: Tick = FRAMES_PER_BLOCK;
    const EPSILON: f64 = 1e-7;
    pub fn from_time(time: f64, rate: f32) -> Tick {
        Tick((time * rate as f64 - Tick::EPSILON).ceil() as u64)
    }

    pub fn advance(&mut self) {
        self.0 += 1;
    }
}
