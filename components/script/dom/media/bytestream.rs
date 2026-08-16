/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Minimal parsers for the byte stream formats registered for Media Source Extensions.
//!
//! These parsers do not decode any media data. They only extract the information the
//! [segment parser loop][loop] needs to drive the observable state of a `SourceBuffer`:
//! which tracks an initialization segment declares, and which presentation interval each
//! media segment covers. The media data itself is handed over to the media backend
//! unmodified.
//!
//! [loop]: https://w3c.github.io/media-source/#sourcebuffer-segment-parser-loop

use std::ops::Range;

/// The byte stream formats we are able to parse.
///
/// <https://w3c.github.io/mse-byte-stream-format-registry/>
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum ByteStreamFormat {
    /// <https://w3c.github.io/mse-byte-stream-format-isobmff/>
    IsoBmff,
    /// <https://w3c.github.io/mse-byte-stream-format-webm/>
    WebM,
}

/// <https://w3c.github.io/media-source/#track-buffer>
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum TrackKind {
    Audio,
    Video,
    Text,
}

/// A track declared by an initialization segment.
#[derive(Clone, Debug, JSTraceable, MallocSizeOf)]
pub(crate) struct TrackDescription {
    pub(crate) id: u32,
    pub(crate) kind: TrackKind,
}

/// The subset of an initialization segment the segment parser loop cares about.
#[derive(Clone, Debug, Default, JSTraceable, MallocSizeOf)]
pub(crate) struct InitializationSegment {
    /// The duration declared by the segment, in seconds, if any.
    pub(crate) duration: Option<f64>,
    pub(crate) tracks: Vec<TrackDescription>,
}

/// Everything a single call to [`ByteStreamParser::parse`] was able to extract.
#[derive(Debug, Default)]
pub(crate) struct ParsedSegments {
    /// The last initialization segment seen, if the append contained one.
    pub(crate) initialization: Option<InitializationSegment>,
    /// The presentation intervals, in seconds, covered by the media segments seen.
    pub(crate) media_ranges: Vec<MediaSegmentRange>,
}

/// The presentation interval a media segment covers for one of its tracks.
#[derive(Debug, PartialEq)]
pub(crate) struct MediaSegmentRange {
    /// The track the interval belongs to, or `None` when it covers every track of the
    /// initialization segment, as a WebM cluster does.
    pub(crate) track: Option<u32>,
    pub(crate) range: Range<f64>,
}

/// An append that could not be parsed, which the caller turns into the
/// [append error algorithm][error].
///
/// [error]: https://w3c.github.io/media-source/#dfn-append-error
#[derive(Debug, PartialEq)]
pub(crate) struct ParseError;

/// The parser state shared by every byte stream format.
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct ByteStreamParser {
    format: ByteStreamFormat,
    /// Bytes of an incomplete structure carried over to the next append.
    pending: Vec<u8>,
    /// Payload bytes we are deliberately not buffering, such as `mdat` contents.
    skip: u64,
    /// Whether a valid initialization segment has been parsed since the last reset.
    have_initialization_segment: bool,
    isobmff: IsoBmffState,
    webm: WebMState,
}

impl ByteStreamParser {
    pub(crate) fn new(format: ByteStreamFormat) -> Self {
        Self {
            format,
            pending: Vec::new(),
            skip: 0,
            have_initialization_segment: false,
            isobmff: IsoBmffState::default(),
            webm: WebMState::default(),
        }
    }

    /// <https://w3c.github.io/media-source/#dfn-reset-parser-state>
    ///
    /// Only the bytes of a partially parsed segment are discarded. The track information
    /// from the initialization segment stays valid, as required by the algorithm.
    pub(crate) fn reset(&mut self) {
        self.pending.clear();
        self.skip = 0;
    }

    pub(crate) fn format(&self) -> ByteStreamFormat {
        self.format
    }

    /// Whether the parser holds the beginning of a structure whose remaining bytes have
    /// not been appended yet, which the specification calls the `PARSING_MEDIA_SEGMENT`
    /// append state.
    ///
    /// <https://w3c.github.io/media-source/#dfn-append-state>
    pub(crate) fn is_parsing_segment(&self) -> bool {
        !self.pending.is_empty() || self.skip > 0
    }

    /// Feeds `data` to the parser and returns everything that could be parsed from the
    /// bytes accumulated so far.
    pub(crate) fn parse(&mut self, data: &[u8]) -> Result<ParsedSegments, ParseError> {
        // Drop the leading bytes of a payload we decided not to buffer.
        let data = if self.skip > 0 {
            let skipped = self.skip.min(data.len() as u64);
            self.skip -= skipped;
            &data[skipped as usize..]
        } else {
            data
        };

        if self.pending.is_empty() {
            // Fast path: parse straight out of the append, and only copy what is left over.
            let mut segments = ParsedSegments::default();
            let consumed = self.parse_buffer(data, &mut segments)?;
            self.pending.extend_from_slice(&data[consumed..]);
            Ok(segments)
        } else {
            self.pending.extend_from_slice(data);
            let mut segments = ParsedSegments::default();
            let buffer = std::mem::take(&mut self.pending);
            let consumed = self.parse_buffer(&buffer, &mut segments)?;
            self.pending.extend_from_slice(&buffer[consumed..]);
            Ok(segments)
        }
    }

    /// Parses as many complete structures as `buffer` holds, returning how many bytes
    /// were consumed.
    fn parse_buffer(
        &mut self,
        buffer: &[u8],
        segments: &mut ParsedSegments,
    ) -> Result<usize, ParseError> {
        match self.format {
            ByteStreamFormat::IsoBmff => self.parse_isobmff(buffer, segments),
            ByteStreamFormat::WebM => self.parse_webm(buffer, segments),
        }
    }
}

// ISO Base Media File Format
//
// <https://w3c.github.io/mse-byte-stream-format-isobmff/>

/// Per-track information an ISO BMFF initialization segment provides, needed to turn the
/// timestamps of later media segments into seconds.
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf)]
struct IsoBmffTrack {
    id: u32,
    timescale: u32,
    default_sample_duration: u32,
}

#[derive(Default, JSTraceable, MallocSizeOf)]
struct IsoBmffState {
    tracks: Vec<IsoBmffTrack>,
}

impl IsoBmffState {
    fn track(&self, id: u32) -> Option<&IsoBmffTrack> {
        self.tracks.iter().find(|track| track.id == id)
    }
}

/// A box header: the payload range within its container plus the four character code.
struct BoxHeader {
    name: [u8; 4],
    /// Offset of the payload relative to the start of the header.
    payload_start: usize,
    /// Total size of the box, header included. `None` when the box extends to the end of
    /// the stream, which ISO BMFF signals with a size of zero.
    total_size: Option<u64>,
}

enum ReadBoxHeader {
    /// The bytes of the header have not all been appended yet.
    Incomplete,
    /// The header does not describe a box, so the stream does not conform to the byte
    /// stream format.
    Invalid,
    Header(BoxHeader),
}

/// Reads the header of the box starting at `data`.
fn read_box_header(data: &[u8]) -> ReadBoxHeader {
    if data.len() < 8 {
        return ReadBoxHeader::Incomplete;
    }

    let size = u32::from_be_bytes(data[0..4].try_into().expect("four bytes")) as u64;
    let name: [u8; 4] = data[4..8].try_into().expect("four bytes");

    // Box types are four printable characters. Checking them keeps a stream of arbitrary
    // bytes from being mistaken for a box whose size we would then wait forever to reach.
    if !name
        .iter()
        .all(|byte| byte.is_ascii_graphic() || *byte == b' ')
    {
        return ReadBoxHeader::Invalid;
    }

    match size {
        // A size of one means the real size is stored in the 64 bit `largesize` field.
        1 => {
            if data.len() < 16 {
                return ReadBoxHeader::Incomplete;
            }
            let size = u64::from_be_bytes(data[8..16].try_into().expect("eight bytes"));
            if size < 16 {
                return ReadBoxHeader::Invalid;
            }
            ReadBoxHeader::Header(BoxHeader {
                name,
                payload_start: 16,
                total_size: Some(size),
            })
        },
        // A size of zero means the box extends to the end of the stream.
        0 => ReadBoxHeader::Header(BoxHeader {
            name,
            payload_start: 8,
            total_size: None,
        }),
        _ if size < 8 => ReadBoxHeader::Invalid,
        _ => ReadBoxHeader::Header(BoxHeader {
            name,
            payload_start: 8,
            total_size: Some(size),
        }),
    }
}

/// Calls `visitor` for every child box of `data`, stopping early if it returns `false`.
fn for_each_box(data: &[u8], mut visitor: impl FnMut(&[u8; 4], &[u8]) -> bool) {
    let mut offset = 0;
    while offset < data.len() {
        let ReadBoxHeader::Header(header) = read_box_header(&data[offset..]) else {
            return;
        };
        let Some(total_size) = header.total_size else {
            // The box runs to the end of the data we have.
            let payload = &data[offset + header.payload_start..];
            visitor(&header.name, payload);
            return;
        };
        let end = offset.saturating_add(total_size as usize);
        if end > data.len() || header.payload_start > total_size as usize {
            return;
        }
        let payload = &data[offset + header.payload_start..end];
        if !visitor(&header.name, payload) {
            return;
        }
        offset = end;
    }
}

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    data.get(offset..offset + 2)
        .and_then(|slice| slice.try_into().ok())
        .map(u16::from_be_bytes)
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    data.get(offset..offset + 4)
        .and_then(|slice| slice.try_into().ok())
        .map(u32::from_be_bytes)
}

fn read_u64(data: &[u8], offset: usize) -> Option<u64> {
    data.get(offset..offset + 8)
        .and_then(|slice| slice.try_into().ok())
        .map(u64::from_be_bytes)
}

/// Returns the version and flags of a full box.
fn read_full_box_header(data: &[u8]) -> Option<(u8, u32)> {
    let value = read_u32(data, 0)?;
    Some(((value >> 24) as u8, value & 0x00ff_ffff))
}

impl ByteStreamParser {
    fn parse_isobmff(
        &mut self,
        buffer: &[u8],
        segments: &mut ParsedSegments,
    ) -> Result<usize, ParseError> {
        let mut offset = 0;

        while offset < buffer.len() {
            let header = match read_box_header(&buffer[offset..]) {
                ReadBoxHeader::Header(header) => header,
                ReadBoxHeader::Incomplete => break,
                ReadBoxHeader::Invalid => return Err(ParseError),
            };
            let Some(total_size) = header.total_size else {
                // Only the final `mdat` of a stream is allowed to have an unknown size, and
                // its payload is of no interest to us.
                return Ok(buffer.len());
            };
            let total_size = total_size as usize;
            if header.payload_start > total_size {
                return Err(ParseError);
            }

            // `mdat` payloads carry the coded frames and can be very large, so they are
            // skipped without ever being copied into the pending buffer.
            if &header.name == b"mdat" {
                let available = buffer.len() - offset;
                if available < total_size {
                    self.skip = (total_size - available) as u64;
                    return Ok(buffer.len());
                }
                offset += total_size;
                continue;
            }

            if buffer.len() - offset < total_size {
                break;
            }

            let payload = &buffer[offset + header.payload_start..offset + total_size];
            match &header.name {
                b"moov" => {
                    let initialization = self.parse_moov(payload).ok_or(ParseError)?;
                    self.have_initialization_segment = true;
                    segments.initialization = Some(initialization);
                },
                b"moof" => {
                    if !self.have_initialization_segment {
                        return Err(ParseError);
                    }
                    segments.media_ranges.extend(self.parse_moof(payload));
                },
                _ => {},
            }

            offset += total_size;
        }

        Ok(offset)
    }

    /// Extracts the tracks and the duration declared by a `moov` box.
    fn parse_moov(&mut self, moov: &[u8]) -> Option<InitializationSegment> {
        let mut movie_timescale = None;
        let mut movie_duration = None;
        let mut tracks = Vec::new();
        let mut parser_tracks = Vec::new();
        // `trex` provides the per-track defaults used by media segments that do not
        // repeat them.
        let mut track_defaults: Vec<(u32, u32)> = Vec::new();

        for_each_box(moov, |name, payload| {
            match name {
                b"mvhd" => {
                    if let Some((version, _)) = read_full_box_header(payload) {
                        let (timescale, duration) = if version == 1 {
                            (
                                read_u32(payload, 20),
                                read_u64(payload, 24).map(|duration| duration as f64),
                            )
                        } else {
                            (
                                read_u32(payload, 12),
                                read_u32(payload, 16).map(|duration| duration as f64),
                            )
                        };
                        movie_timescale = timescale;
                        movie_duration = duration;
                    }
                },
                b"mvex" => {
                    for_each_box(payload, |name, payload| {
                        if name == b"trex" &&
                            let Some(id) = read_u32(payload, 4) &&
                            let Some(default_sample_duration) = read_u32(payload, 12)
                        {
                            track_defaults.push((id, default_sample_duration));
                        }
                        true
                    });
                },
                b"trak" => {
                    if let Some((track, kind)) = parse_trak(payload) {
                        tracks.push(TrackDescription { id: track.id, kind });
                        parser_tracks.push(track);
                    }
                },
                _ => {},
            }
            true
        });

        if tracks.is_empty() {
            return None;
        }

        for track in parser_tracks.iter_mut() {
            if let Some((_, default_sample_duration)) =
                track_defaults.iter().find(|(id, _)| *id == track.id)
            {
                track.default_sample_duration = *default_sample_duration;
            }
        }
        self.isobmff.tracks = parser_tracks;

        // An unset or zero movie duration means the duration is unknown, which is how
        // live streams and most media segments-only initialization segments are authored.
        let duration = match (movie_timescale, movie_duration) {
            (Some(timescale), Some(duration)) if timescale > 0 && duration > 0. => {
                Some(duration / timescale as f64)
            },
            _ => None,
        };

        Some(InitializationSegment { duration, tracks })
    }

    /// Computes the presentation interval a `moof` box covers for each of its tracks.
    fn parse_moof(&self, moof: &[u8]) -> Vec<MediaSegmentRange> {
        let mut ranges = Vec::new();

        for_each_box(moof, |name, payload| {
            if name == b"traf" &&
                let Some(range) = self.parse_traf(payload)
            {
                ranges.push(range);
            }
            true
        });

        ranges
    }

    fn parse_traf(&self, traf: &[u8]) -> Option<MediaSegmentRange> {
        let mut track = None;
        let mut default_sample_duration = None;
        let mut base_media_decode_time = None;
        let mut total_duration: u64 = 0;
        let mut have_trun = false;

        for_each_box(traf, |name, payload| {
            match name {
                b"tfhd" => {
                    if let Some((_, flags)) = read_full_box_header(payload) &&
                        let Some(track_id) = read_u32(payload, 4)
                    {
                        track = self.isobmff.track(track_id).copied();

                        // The optional fields of `tfhd` are laid out in flag order, so the
                        // offset of `default_sample_duration` depends on which of the
                        // preceding fields are present.
                        let mut offset = 8;
                        if flags & 0x00_0001 != 0 {
                            // base_data_offset
                            offset += 8;
                        }
                        if flags & 0x00_0002 != 0 {
                            // sample_description_index
                            offset += 4;
                        }
                        if flags & 0x00_0008 != 0 {
                            default_sample_duration = read_u32(payload, offset);
                        }
                    }
                },
                b"tfdt" => {
                    if let Some((version, _)) = read_full_box_header(payload) {
                        base_media_decode_time = if version == 1 {
                            read_u64(payload, 4)
                        } else {
                            read_u32(payload, 4).map(u64::from)
                        };
                    }
                },
                b"trun" => {
                    if let Some((_, flags)) = read_full_box_header(payload) &&
                        let Some(sample_count) = read_u32(payload, 4)
                    {
                        have_trun = true;
                        total_duration = total_duration.saturating_add(trun_duration(
                            payload,
                            flags,
                            sample_count,
                            default_sample_duration
                                .or_else(|| track.map(|track| track.default_sample_duration)),
                        ));
                    }
                },
                _ => {},
            }
            true
        });

        let track = track?;
        if track.timescale == 0 || !have_trun {
            return None;
        }

        let timescale = track.timescale as f64;
        let start = base_media_decode_time.unwrap_or(0) as f64 / timescale;
        let end = start + total_duration as f64 / timescale;

        Some(MediaSegmentRange {
            track: Some(track.id),
            range: start..end,
        })
    }
}

/// Sums the durations of the samples described by a `trun` box.
fn trun_duration(
    trun: &[u8],
    flags: u32,
    sample_count: u32,
    default_sample_duration: Option<u32>,
) -> u64 {
    const DATA_OFFSET_PRESENT: u32 = 0x00_0001;
    const FIRST_SAMPLE_FLAGS_PRESENT: u32 = 0x00_0004;
    const SAMPLE_DURATION_PRESENT: u32 = 0x00_0100;
    const SAMPLE_SIZE_PRESENT: u32 = 0x00_0200;
    const SAMPLE_FLAGS_PRESENT: u32 = 0x00_0400;
    const SAMPLE_COMPOSITION_TIME_OFFSET_PRESENT: u32 = 0x00_0800;

    // When the samples do not carry their own duration, they all last the default.
    if flags & SAMPLE_DURATION_PRESENT == 0 {
        return u64::from(default_sample_duration.unwrap_or(0))
            .saturating_mul(u64::from(sample_count));
    }

    let mut offset = 8;
    if flags & DATA_OFFSET_PRESENT != 0 {
        offset += 4;
    }
    if flags & FIRST_SAMPLE_FLAGS_PRESENT != 0 {
        offset += 4;
    }

    let mut entry_size = 4;
    if flags & SAMPLE_SIZE_PRESENT != 0 {
        entry_size += 4;
    }
    if flags & SAMPLE_FLAGS_PRESENT != 0 {
        entry_size += 4;
    }
    if flags & SAMPLE_COMPOSITION_TIME_OFFSET_PRESENT != 0 {
        entry_size += 4;
    }

    let mut total = 0u64;
    for _ in 0..sample_count {
        let Some(duration) = read_u32(trun, offset) else {
            break;
        };
        total = total.saturating_add(u64::from(duration));
        offset += entry_size;
    }
    total
}

/// Reads the track identifier, timescale and media kind out of a `trak` box.
fn parse_trak(trak: &[u8]) -> Option<(IsoBmffTrack, TrackKind)> {
    let mut id = None;
    let mut timescale = None;
    let mut kind = None;

    for_each_box(trak, |name, payload| {
        match name {
            b"tkhd" => {
                if let Some((version, _)) = read_full_box_header(payload) {
                    id = if version == 1 {
                        read_u32(payload, 20)
                    } else {
                        read_u32(payload, 12)
                    };
                }
            },
            b"mdia" => {
                for_each_box(payload, |name, payload| {
                    match name {
                        b"mdhd" => {
                            if let Some((version, _)) = read_full_box_header(payload) {
                                timescale = if version == 1 {
                                    read_u32(payload, 20)
                                } else {
                                    read_u32(payload, 12)
                                };
                            }
                        },
                        b"hdlr" => {
                            kind = match payload.get(8..12) {
                                Some(b"vide") => Some(TrackKind::Video),
                                Some(b"soun") => Some(TrackKind::Audio),
                                Some(b"text") | Some(b"subt") | Some(b"sbtl") => {
                                    Some(TrackKind::Text)
                                },
                                _ => None,
                            };
                        },
                        _ => {},
                    }
                    true
                });
            },
            _ => {},
        }
        true
    });

    Some((
        IsoBmffTrack {
            id: id?,
            timescale: timescale?,
            default_sample_duration: 0,
        },
        kind?,
    ))
}

// WebM
//
// <https://w3c.github.io/mse-byte-stream-format-webm/>

const EBML_HEADER: u64 = 0x1a45_dfa3;
const SEGMENT: u64 = 0x1853_8067;
const INFO: u64 = 0x1549_a966;
const TIMECODE_SCALE: u64 = 0x2ad7_b1;
const DURATION: u64 = 0x4489;
const TRACKS: u64 = 0x1654_ae6b;
const TRACK_ENTRY: u64 = 0xae;
const TRACK_NUMBER: u64 = 0xd7;
const TRACK_TYPE: u64 = 0x83;
const DEFAULT_DURATION: u64 = 0x23e3_83;
const CLUSTER: u64 = 0x1f43_b675;
const TIMECODE: u64 = 0xe7;
const SIMPLE_BLOCK: u64 = 0xa3;
const BLOCK_GROUP: u64 = 0xa0;
const BLOCK: u64 = 0xa1;

/// The nanoseconds per timecode unit assumed when a segment omits `TimecodeScale`.
const DEFAULT_TIMECODE_SCALE: u64 = 1_000_000;

/// The elements that may appear either at the top level of a WebM stream or directly
/// inside its `Segment`, whose children are parsed at the top level here.
///
/// <https://www.matroska.org/technical/elements.html>
const TOP_LEVEL_ELEMENTS: &[u64] = &[
    EBML_HEADER,
    SEGMENT,
    INFO,
    TRACKS,
    CLUSTER,
    // SeekHead
    0x114d_9b74,
    // Cues
    0x1c53_bb6b,
    // Chapters
    0x1043_a770,
    // Tags
    0x1254_c367,
    // Attachments
    0x1941_a469,
    // Void
    0xec,
    // CRC-32
    0xbf,
];

#[derive(JSTraceable, MallocSizeOf)]
struct WebMState {
    timecode_scale: u64,
    /// The `DefaultDuration` of the track with the finest granularity, in nanoseconds,
    /// used to extend the last block of a cluster to its real end.
    default_duration: Option<u64>,
}

impl Default for WebMState {
    fn default() -> Self {
        Self {
            timecode_scale: DEFAULT_TIMECODE_SCALE,
            default_duration: None,
        }
    }
}

/// An EBML variable size integer.
struct VarInt {
    value: u64,
    length: usize,
    /// Whether every value bit was set, which EBML uses to mean "unknown size".
    unknown: bool,
}

/// Reads a variable size integer, optionally keeping the length marker as element
/// identifiers require.
fn read_vint(data: &[u8], keep_marker: bool) -> Option<VarInt> {
    let first = *data.first()?;
    if first == 0 {
        // Lengths above eight bytes are not valid in WebM.
        return None;
    }
    let length = first.leading_zeros() as usize + 1;
    if data.len() < length {
        return None;
    }

    let mut value = if keep_marker {
        u64::from(first)
    } else {
        // The length marker takes `length` bits of the first byte, so an eight byte
        // integer carries none of its value there.
        let mask = if length >= 8 { 0 } else { 0xffu8 >> length };
        u64::from(first & mask)
    };
    for byte in &data[1..length] {
        value = (value << 8) | u64::from(*byte);
    }

    // With the marker stripped, an all-ones value means the size is unknown.
    let unknown = !keep_marker && value == (1u64 << (7 * length)) - 1;

    Some(VarInt {
        value,
        length,
        unknown,
    })
}

struct EbmlHeader {
    id: u64,
    payload_start: usize,
    /// `None` for elements of unknown size.
    payload_size: Option<u64>,
}

fn read_ebml_header(data: &[u8]) -> Option<EbmlHeader> {
    let id = read_vint(data, /* keep_marker */ true)?;
    let size = read_vint(&data[id.length..], /* keep_marker */ false)?;
    Some(EbmlHeader {
        id: id.value,
        payload_start: id.length + size.length,
        payload_size: (!size.unknown).then_some(size.value),
    })
}

/// Reads an unsigned integer stored in a big endian element of up to eight bytes.
fn read_ebml_uint(data: &[u8]) -> Option<u64> {
    if data.is_empty() || data.len() > 8 {
        return None;
    }
    Some(
        data.iter()
            .fold(0u64, |value, byte| (value << 8) | u64::from(*byte)),
    )
}

fn read_ebml_float(data: &[u8]) -> Option<f64> {
    match data.len() {
        4 => Some(f32::from_be_bytes(data.try_into().ok()?) as f64),
        8 => Some(f64::from_be_bytes(data.try_into().ok()?)),
        _ => None,
    }
}

impl ByteStreamParser {
    fn parse_webm(
        &mut self,
        buffer: &[u8],
        segments: &mut ParsedSegments,
    ) -> Result<usize, ParseError> {
        let mut offset = 0;
        let mut initialization: Option<InitializationSegment> = None;

        while offset < buffer.len() {
            let Some(header) = read_ebml_header(&buffer[offset..]) else {
                break;
            };

            // Anything that is not one of the elements a WebM stream may carry at this
            // level means the stream does not conform to the byte stream format. Without
            // this check a run of arbitrary bytes would read as an element whose size we
            // would then wait forever to reach.
            if !TOP_LEVEL_ELEMENTS.contains(&header.id) {
                return Err(ParseError);
            }

            // `Segment` is a master element that usually has an unknown size and always
            // wraps the whole stream, so its children are parsed as if they were at the
            // top level.
            if header.id == SEGMENT {
                offset += header.payload_start;
                continue;
            }

            let Some(payload_size) = header.payload_size else {
                // Only `Segment` is allowed an unknown size in the WebM byte stream format.
                return Err(ParseError);
            };
            let total_size = header.payload_start.saturating_add(payload_size as usize);
            if buffer.len() - offset < total_size {
                break;
            }
            let payload = &buffer[offset + header.payload_start..offset + total_size];

            match header.id {
                EBML_HEADER => {
                    self.webm = WebMState::default();
                },
                INFO => {
                    let info = self.parse_webm_info(payload);
                    initialization
                        .get_or_insert_with(InitializationSegment::default)
                        .duration = info;
                },
                TRACKS => {
                    let tracks = self.parse_webm_tracks(payload);
                    if tracks.is_empty() {
                        return Err(ParseError);
                    }
                    self.have_initialization_segment = true;
                    initialization
                        .get_or_insert_with(InitializationSegment::default)
                        .tracks = tracks;
                },
                CLUSTER => {
                    if !self.have_initialization_segment {
                        return Err(ParseError);
                    }
                    if let Some(range) = self.parse_webm_cluster(payload) {
                        segments.media_ranges.push(range);
                    }
                },
                _ => {},
            }

            offset += total_size;
        }

        if initialization.is_some() {
            segments.initialization = initialization;
        }

        Ok(offset)
    }

    /// Reads `TimecodeScale` and `Duration` out of an `Info` element.
    fn parse_webm_info(&mut self, info: &[u8]) -> Option<f64> {
        let mut duration = None;

        for_each_ebml_element(info, |id, payload| {
            match id {
                TIMECODE_SCALE => {
                    if let Some(scale) = read_ebml_uint(payload).filter(|scale| *scale > 0) {
                        self.webm.timecode_scale = scale;
                    }
                },
                DURATION => {
                    duration = read_ebml_float(payload).filter(|duration| *duration > 0.);
                },
                _ => {},
            }
            true
        });

        // `Duration` is expressed in timecode scale units.
        duration.map(|duration| duration * self.webm.timecode_scale as f64 / 1e9)
    }

    fn parse_webm_tracks(&mut self, tracks: &[u8]) -> Vec<TrackDescription> {
        let mut descriptions = Vec::new();
        let mut default_duration = None;

        for_each_ebml_element(tracks, |id, payload| {
            if id != TRACK_ENTRY {
                return true;
            }

            let mut number = None;
            let mut kind = None;
            for_each_ebml_element(payload, |id, payload| {
                match id {
                    TRACK_NUMBER => number = read_ebml_uint(payload),
                    TRACK_TYPE => {
                        // <https://www.matroska.org/technical/elements.html>
                        kind = match read_ebml_uint(payload) {
                            Some(1) => Some(TrackKind::Video),
                            Some(2) => Some(TrackKind::Audio),
                            Some(0x11) => Some(TrackKind::Text),
                            _ => None,
                        };
                    },
                    DEFAULT_DURATION => {
                        if let Some(duration) = read_ebml_uint(payload).filter(|d| *d > 0) {
                            default_duration = Some(
                                default_duration
                                    .map_or(duration, |current: u64| current.min(duration)),
                            );
                        }
                    },
                    _ => {},
                }
                true
            });

            if let (Some(number), Some(kind)) = (number, kind) {
                descriptions.push(TrackDescription {
                    id: number as u32,
                    kind,
                });
            }
            true
        });

        self.webm.default_duration = default_duration;
        descriptions
    }

    /// Computes the presentation interval covered by a `Cluster` element. A cluster
    /// interleaves every track of the segment over the same interval.
    fn parse_webm_cluster(&self, cluster: &[u8]) -> Option<MediaSegmentRange> {
        let mut timecode = None;
        // The largest block timecode relative to the cluster timecode.
        let mut last_relative = 0i64;

        for_each_ebml_element(cluster, |id, payload| {
            match id {
                TIMECODE => timecode = read_ebml_uint(payload),
                SIMPLE_BLOCK => {
                    if let Some(relative) = read_block_relative_timecode(payload) {
                        last_relative = last_relative.max(i64::from(relative));
                    }
                },
                BLOCK_GROUP => {
                    for_each_ebml_element(payload, |id, payload| {
                        if id == BLOCK &&
                            let Some(relative) = read_block_relative_timecode(payload)
                        {
                            last_relative = last_relative.max(i64::from(relative));
                        }
                        true
                    });
                },
                _ => {},
            }
            true
        });

        let timecode = timecode?;
        let scale = self.webm.timecode_scale as f64;
        let start = timecode as f64 * scale / 1e9;
        let last_frame = (timecode as f64 + last_relative as f64) * scale / 1e9;
        // Extend the interval past the last block so that a cluster reports the time it
        // actually covers rather than the presentation time of its final frame.
        let end = last_frame + self.webm.default_duration.unwrap_or(0) as f64 / 1e9;

        Some(MediaSegmentRange {
            track: None,
            range: start..end.max(start),
        })
    }
}

/// Reads the signed timecode a block stores relative to its cluster.
fn read_block_relative_timecode(block: &[u8]) -> Option<i16> {
    // A block starts with the track number as a variable size integer, followed by the
    // relative timecode as a big endian signed 16 bit integer.
    let track_number = read_vint(block, /* keep_marker */ false)?;
    read_u16(block, track_number.length).map(|value| value as i16)
}

/// Calls `visitor` for every child element of `data`, stopping early if it returns `false`.
fn for_each_ebml_element(data: &[u8], mut visitor: impl FnMut(u64, &[u8]) -> bool) {
    let mut offset = 0;
    while offset < data.len() {
        let Some(header) = read_ebml_header(&data[offset..]) else {
            return;
        };
        let Some(payload_size) = header.payload_size else {
            return;
        };
        let end = offset.saturating_add(header.payload_start.saturating_add(payload_size as usize));
        if end > data.len() {
            return;
        }
        if !visitor(header.id, &data[offset + header.payload_start..end]) {
            return;
        }
        offset = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an ISO BMFF box with the given four character code and payload.
    fn boxed(name: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut data = Vec::with_capacity(8 + payload.len());
        data.extend_from_slice(&((8 + payload.len()) as u32).to_be_bytes());
        data.extend_from_slice(name);
        data.extend_from_slice(payload);
        data
    }

    fn full_box(version: u8, flags: u32) -> Vec<u8> {
        (((version as u32) << 24) | flags).to_be_bytes().to_vec()
    }

    /// A `moov` with a single 25 fps video track using a timescale of 1000.
    fn video_moov(movie_duration: u32) -> Vec<u8> {
        let mut mvhd = full_box(0, 0);
        mvhd.extend_from_slice(&0u32.to_be_bytes()); // creation_time
        mvhd.extend_from_slice(&0u32.to_be_bytes()); // modification_time
        mvhd.extend_from_slice(&1000u32.to_be_bytes()); // timescale
        mvhd.extend_from_slice(&movie_duration.to_be_bytes());

        let mut tkhd = full_box(0, 0);
        tkhd.extend_from_slice(&0u32.to_be_bytes()); // creation_time
        tkhd.extend_from_slice(&0u32.to_be_bytes()); // modification_time
        tkhd.extend_from_slice(&1u32.to_be_bytes()); // track_ID

        let mut mdhd = full_box(0, 0);
        mdhd.extend_from_slice(&0u32.to_be_bytes()); // creation_time
        mdhd.extend_from_slice(&0u32.to_be_bytes()); // modification_time
        mdhd.extend_from_slice(&1000u32.to_be_bytes()); // timescale

        let mut hdlr = full_box(0, 0);
        hdlr.extend_from_slice(&0u32.to_be_bytes()); // pre_defined
        hdlr.extend_from_slice(b"vide");

        let mdia = boxed(
            b"mdia",
            &[boxed(b"mdhd", &mdhd), boxed(b"hdlr", &hdlr)].concat(),
        );
        let trak = boxed(b"trak", &[boxed(b"tkhd", &tkhd), mdia].concat());

        boxed(b"moov", &[boxed(b"mvhd", &mvhd), trak].concat())
    }

    /// A `moof` starting at `base_time` with `samples` samples of `duration` each.
    fn video_moof(base_time: u32, samples: u32, duration: u32) -> Vec<u8> {
        let mut tfhd = full_box(0, 0);
        tfhd.extend_from_slice(&1u32.to_be_bytes()); // track_ID

        let mut tfdt = full_box(0, 0);
        tfdt.extend_from_slice(&base_time.to_be_bytes());

        // sample-duration-present
        let mut trun = full_box(0, 0x00_0100);
        trun.extend_from_slice(&samples.to_be_bytes());
        for _ in 0..samples {
            trun.extend_from_slice(&duration.to_be_bytes());
        }

        let traf = boxed(
            b"traf",
            &[
                boxed(b"tfhd", &tfhd),
                boxed(b"tfdt", &tfdt),
                boxed(b"trun", &trun),
            ]
            .concat(),
        );
        boxed(b"moof", &traf)
    }

    #[test]
    fn isobmff_initialization_segment_reports_tracks_and_duration() {
        let mut parser = ByteStreamParser::new(ByteStreamFormat::IsoBmff);
        let data = [boxed(b"ftyp", b"isom"), video_moov(10_000)].concat();

        let segments = parser.parse(&data).unwrap();
        let initialization = segments.initialization.expect("initialization segment");

        assert_eq!(initialization.duration, Some(10.));
        assert_eq!(initialization.tracks.len(), 1);
        assert_eq!(initialization.tracks[0].id, 1);
        assert_eq!(initialization.tracks[0].kind, TrackKind::Video);
        assert!(parser.have_initialization_segment);
    }

    #[test]
    fn isobmff_media_segment_reports_presentation_interval() {
        let mut parser = ByteStreamParser::new(ByteStreamFormat::IsoBmff);
        parser
            .parse(&[boxed(b"ftyp", b"isom"), video_moov(0)].concat())
            .unwrap();

        // Ten samples of 40ms starting at two seconds.
        let segments = parser
            .parse(&[video_moof(2000, 10, 40), boxed(b"mdat", &[0; 64])].concat())
            .unwrap();

        assert_eq!(
            segments.media_ranges,
            vec![MediaSegmentRange {
                track: Some(1),
                range: 2.0..2.4,
            }]
        );
    }

    #[test]
    fn isobmff_boxes_split_across_appends_are_parsed() {
        let mut parser = ByteStreamParser::new(ByteStreamFormat::IsoBmff);
        let data = [boxed(b"ftyp", b"isom"), video_moov(4_000)].concat();

        let (first, second) = data.split_at(data.len() / 2);
        assert!(parser.parse(first).unwrap().initialization.is_none());

        let segments = parser.parse(second).unwrap();
        assert_eq!(
            segments
                .initialization
                .expect("initialization segment")
                .duration,
            Some(4.)
        );
    }

    #[test]
    fn isobmff_large_mdat_payloads_are_not_buffered() {
        let mut parser = ByteStreamParser::new(ByteStreamFormat::IsoBmff);
        parser
            .parse(&[boxed(b"ftyp", b"isom"), video_moov(0)].concat())
            .unwrap();

        let mdat = boxed(b"mdat", &[0; 4096]);
        // Feed the `mdat` in two pieces so that the skip counter is exercised.
        let (first, second) = mdat.split_at(1000);
        parser.parse(first).unwrap();
        parser.parse(second).unwrap();
        assert!(parser.pending.is_empty());

        let segments = parser.parse(&video_moof(0, 5, 100)).unwrap();
        assert_eq!(
            segments.media_ranges,
            vec![MediaSegmentRange {
                track: Some(1),
                range: 0.0..0.5,
            }]
        );
    }

    #[test]
    fn isobmff_media_segment_without_initialization_segment_is_an_error() {
        let mut parser = ByteStreamParser::new(ByteStreamFormat::IsoBmff);
        assert!(parser.parse(&video_moof(0, 1, 100)).is_err());
    }

    #[test]
    fn isobmff_arbitrary_bytes_are_an_error() {
        let mut parser = ByteStreamParser::new(ByteStreamFormat::IsoBmff);
        let garbage: Vec<u8> = (0..64u8).collect();
        assert!(parser.parse(&garbage).is_err());
    }

    #[test]
    fn isobmff_box_smaller_than_its_header_is_an_error() {
        let mut parser = ByteStreamParser::new(ByteStreamFormat::IsoBmff);
        let mut data = 5u32.to_be_bytes().to_vec();
        data.extend_from_slice(b"ftyp");
        assert!(parser.parse(&data).is_err());
    }

    /// Builds an EBML element with the given identifier and payload.
    fn element(id: &[u8], payload: &[u8]) -> Vec<u8> {
        let mut data = Vec::with_capacity(id.len() + 8 + payload.len());
        data.extend_from_slice(id);
        // Encode the size as an eight byte variable size integer for simplicity.
        let mut size = (payload.len() as u64).to_be_bytes();
        size[0] |= 0x01;
        data.extend_from_slice(&size);
        data.extend_from_slice(payload);
        data
    }

    fn webm_initialization_segment(duration_ms: f64) -> Vec<u8> {
        let info = element(
            &[0x15, 0x49, 0xa9, 0x66],
            &[
                element(&[0x2a, 0xd7, 0xb1], &1_000_000u32.to_be_bytes()),
                element(&[0x44, 0x89], &(duration_ms as f64).to_be_bytes()),
            ]
            .concat(),
        );
        let track_entry = element(
            &[0xae],
            &[
                element(&[0xd7], &[1]),
                element(&[0x83], &[2]),
                element(&[0x23, 0xe3, 0x83], &20_000_000u32.to_be_bytes()),
            ]
            .concat(),
        );
        let tracks = element(&[0x16, 0x54, 0xae, 0x6b], &track_entry);

        [
            element(&[0x1a, 0x45, 0xdf, 0xa3], &[]),
            // `Segment` with an unknown size, as authored by most muxers.
            vec![0x18, 0x53, 0x80, 0x67, 0xff],
            info,
            tracks,
        ]
        .concat()
    }

    #[test]
    fn webm_initialization_segment_reports_tracks_and_duration() {
        let mut parser = ByteStreamParser::new(ByteStreamFormat::WebM);
        let segments = parser.parse(&webm_initialization_segment(5000.)).unwrap();
        let initialization = segments.initialization.expect("initialization segment");

        // A `Duration` of 5000 with a timecode scale of a millisecond is five seconds.
        assert_eq!(initialization.duration, Some(5.));
        assert_eq!(initialization.tracks.len(), 1);
        assert_eq!(initialization.tracks[0].kind, TrackKind::Audio);
    }

    #[test]
    fn webm_arbitrary_bytes_are_an_error() {
        let mut parser = ByteStreamParser::new(ByteStreamFormat::WebM);
        let garbage: Vec<u8> = (0..64u8).map(|byte| byte.wrapping_add(0x40)).collect();
        assert!(parser.parse(&garbage).is_err());
    }

    #[test]
    fn webm_cluster_reports_presentation_interval() {
        let mut parser = ByteStreamParser::new(ByteStreamFormat::WebM);
        parser.parse(&webm_initialization_segment(5000.)).unwrap();

        // A block for track one, 100ms after the cluster timecode of one second.
        let mut block = vec![0x81];
        block.extend_from_slice(&100i16.to_be_bytes());
        block.push(0x80); // flags
        let cluster = element(
            &[0x1f, 0x43, 0xb6, 0x75],
            &[
                element(&[0xe7], &1000u16.to_be_bytes()),
                element(&[0xa3], &block),
            ]
            .concat(),
        );

        let segments = parser.parse(&cluster).unwrap();
        assert_eq!(segments.media_ranges.len(), 1);
        let range = &segments.media_ranges[0].range;
        assert_eq!(range.start, 1.0);
        // The last block starts at 1.1s and lasts for the default duration of 20ms.
        assert!((range.end - 1.12).abs() < f64::EPSILON * 8.);
    }
}
