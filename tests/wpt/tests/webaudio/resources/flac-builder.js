/**
 * In-memory FLAC file synthesizer for Web Audio decode tests.
 *
 * buildFlac(options) returns an ArrayBuffer holding a spec-valid FLAC stream made
 * of silent CONSTANT-subframe frames, so a test can synthesize FLAC with a chosen
 * sample rate, block size, frame count and channel layout without shipping a binary
 * asset. Samples are always 16-bit (BITS_PER_SAMPLE).
 */

(function () {
  "use strict";

  // "fLaC" stream marker that opens every FLAC file.
  const FLAC_STREAM_MARKER = [0x66, 0x4c, 0x61, 0x43];
  // Metadata block header byte: last-metadata-block flag (0x80) | block type 0 (STREAMINFO).
  const METADATA_BLOCK_HEADER_STREAMINFO_LAST = 0x80;
  // STREAMINFO payload length in bytes (fixed at 34).
  const STREAMINFO_LENGTH = 34;
  // Length of the trailing MD5 signature field (left zero = unset).
  const MD5_SIGNATURE_LENGTH = 16;
  // Frame sync: 14-bit sync code with fixed blocking strategy.
  const FRAME_SYNC = [0xff, 0xf8];
  // Frame-header block-size code (top nibble 0b0111): a 16-bit (blockSize-1) follows the header.
  const BLOCKSIZE_CODE_16BIT_MINUS1 = 0x70;
  // Sample-size nibble for 16-bit samples in the frame-header channel/bps byte.
  const SAMPLE_SIZE_CODE_16BIT = 0x08;
  // Subframe header byte: CONSTANT subframe type with 0 wasted bits.
  const SUBFRAME_HEADER_CONSTANT = 0x00;
  // Generator polynomials for the header CRC-8 and the frame CRC-16.
  const CRC8_POLY = 0x07;
  const CRC16_POLY = 0x8005;
  // Samples are emitted as 16-bit constants. To support other depths in the
  // future, make this a per-call option and update three byte-packed sites: the
  // (bps - 1) field in buildStreamInfo, the sample-size nibble in buildFrame's
  // header (SAMPLE_SIZE_CODE_16BIT), and the per-sample byte width written into
  // each CONSTANT subframe (currently a fixed big-endian 16-bit value).
  const BITS_PER_SAMPLE = 16;
  // FLAC independent-channel assignments encode 1..8 channels as (channels - 1).
  const MAX_CHANNELS = 8;
  // STREAMINFO sample-rate code (index into the FLAC fixed sample-rate table).
  const SAMPLE_RATE_CODES = {
    88200: 1,
    176400: 2,
    192000: 3,
    8000: 4,
    16000: 5,
    22050: 6,
    24000: 7,
    32000: 8,
    44100: 9,
    48000: 10,
    96000: 11,
  };
  // Splits the 36-bit STREAMINFO total-samples field.
  const TWO_POW_32 = 2 ** 32;

  // CRC-8 (polynomial 0x07, MSB-first) over the frame header bytes; the result
  // is the last byte of every FLAC frame header.
  function crc8(bytes) {
    let c = 0;
    for (const b of bytes) {
      c ^= b;
      for (let i = 0; i < 8; i++) {
        c = c & 0x80 ? ((c << 1) ^ CRC8_POLY) & 0xff : (c << 1) & 0xff;
      }
    }
    return c;
  }

  // CRC-16 (polynomial 0x8005, MSB-first) over the whole frame up to the
  // checksum; the result is the two trailing bytes of every FLAC frame.
  function crc16(bytes) {
    let c = 0;
    for (const b of bytes) {
      c ^= b << 8;
      for (let i = 0; i < 8; i++) {
        c = c & 0x8000 ? ((c << 1) ^ CRC16_POLY) & 0xffff : (c << 1) & 0xffff;
      }
    }
    return c;
  }

  // Encodes a frame number as FLAC's UTF-8-like variable-length integer (the
  // "coded number" in a fixed-blocksize frame header): 1-4 bytes whose length
  // grows with the value's magnitude, mirroring UTF-8 byte sequences.
  function utf8FrameNumber(n) {
    if (n < 0x80) {
      return [n];
    }
    if (n < 0x800) {
      return [0xc0 | (n >> 6), 0x80 | (n & 0x3f)];
    }
    if (n < 0x10000) {
      return [0xe0 | (n >> 12), 0x80 | ((n >> 6) & 0x3f), 0x80 | (n & 0x3f)];
    }
    return [
      0xf0 | (n >> 18),
      0x80 | ((n >> 12) & 0x3f),
      0x80 | ((n >> 6) & 0x3f),
      0x80 | (n & 0x3f),
    ];
  }

  function buildStreamInfo(blockSize, sampleRate, channels, totalSamples) {
    // The packed field after the block-size and frame-size fields holds, in order:
    //   20 bits sample rate | 3 bits (channels - 1) | 5 bits (bps - 1) | 36 bits total samples.
    const bpsMinus1 = BITS_PER_SAMPLE - 1;
    const totalHi = Math.floor(totalSamples / TWO_POW_32); // top 4 bits of the 36-bit total
    const totalLo = totalSamples % TWO_POW_32; // low 32 bits of the total
    return [
      ...FLAC_STREAM_MARKER,
      METADATA_BLOCK_HEADER_STREAMINFO_LAST,
      0x00,
      0x00,
      STREAMINFO_LENGTH,
      (blockSize >> 8) & 0xff,
      blockSize & 0xff, // min block size
      (blockSize >> 8) & 0xff,
      blockSize & 0xff, // max block size
      0x00,
      0x00,
      0x00, // min frame size (unknown)
      0x00,
      0x00,
      0x00, // max frame size (unknown)
      (sampleRate >> 12) & 0xff,
      (sampleRate >> 4) & 0xff,
      ((sampleRate & 0x0f) << 4) |
        (((channels - 1) & 0x07) << 1) |
        ((bpsMinus1 >> 4) & 0x01),
      ((bpsMinus1 & 0x0f) << 4) | (totalHi & 0x0f),
      (totalLo >>> 24) & 0xff,
      (totalLo >>> 16) & 0xff,
      (totalLo >>> 8) & 0xff,
      totalLo & 0xff,
      ...new Array(MD5_SIGNATURE_LENGTH).fill(0),
    ];
  }

  function buildFrame(
    frameIndex,
    blockSize,
    sampleRate,
    channels,
    sampleValue
  ) {
    const blockSizeMinus1 = blockSize - 1;
    const header = [
      ...FRAME_SYNC,
      BLOCKSIZE_CODE_16BIT_MINUS1 | SAMPLE_RATE_CODES[sampleRate],
      (((channels - 1) & 0x07) << 4) | SAMPLE_SIZE_CODE_16BIT,
      ...utf8FrameNumber(frameIndex),
      (blockSizeMinus1 >> 8) & 0xff,
      blockSizeMinus1 & 0xff,
    ];
    header.push(crc8(header));

    const frame = header.slice();
    const sampleHi = (sampleValue >> 8) & 0xff;
    const sampleLo = sampleValue & 0xff;
    for (let c = 0; c < channels; c++) {
      frame.push(SUBFRAME_HEADER_CONSTANT, sampleHi, sampleLo);
    }
    const crc = crc16(frame);
    frame.push((crc >> 8) & 0xff, crc & 0xff);
    return frame;
  }

  /**
   * options:
   *   sampleRate   : FLAC fixed-table rate (8000/16000/22050/24000/32000/44100/
   *                  48000/88200/96000/176400/192000)                    [required]
   *   blockSize    : samples per FLAC frame (e.g. 65535)                  [required]
   *   frameCount   : number of FLAC frames to emit                        [required]
   *   totalSamples : value written to the STREAMINFO total-samples field
   *                  (defaults to blockSize * frameCount; kept separate so a
   *                  test can declare an oversized total on purpose)      [optional]
   *   channels     : 1..8 independent channels                           [required]
   *   sampleValue  : constant 16-bit sample emitted in every subframe
   *                  (default 0 = silence)                                [optional]
   * returns: ArrayBuffer of the encoded FLAC file.
   */
  function buildFlac(options) {
    const {
      sampleRate,
      blockSize,
      frameCount,
      totalSamples = blockSize * frameCount,
      channels,
      sampleValue = 0,
    } = options;

    if (!(sampleRate in SAMPLE_RATE_CODES)) {
      throw new Error(
        `buildFlac: sampleRate ${sampleRate} is not a FLAC fixed-table rate`
      );
    }
    if (
      !Number.isInteger(channels) ||
      channels < 1 ||
      channels > MAX_CHANNELS
    ) {
      throw new Error(
        `buildFlac: channels must be an integer in 1..${MAX_CHANNELS}`
      );
    }

    const streamInfo = buildStreamInfo(
      blockSize,
      sampleRate,
      channels,
      totalSamples
    );
    let length = streamInfo.length;
    const frames = [];
    for (let i = 0; i < frameCount; i++) {
      const frame = buildFrame(i, blockSize, sampleRate, channels, sampleValue);
      frames.push(frame);
      length += frame.length;
    }

    const out = new Uint8Array(length);
    out.set(streamInfo, 0);
    let offset = streamInfo.length;
    for (const frame of frames) {
      out.set(frame, offset);
      offset += frame.length;
    }
    return out.buffer;
  }

  window.buildFlac = buildFlac;
})();
