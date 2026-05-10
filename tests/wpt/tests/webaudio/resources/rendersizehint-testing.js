// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

const OFFLINE_CONTEXT_LENGTH = 1000;

const keywordCases = [
  { hint: undefined, description: 'no hint' },
  { hint: 'default', description: '"default"' },
  { hint: 'hardware', description: '"hardware"' },
];

// Comprehensive numeric valid combinations (honored exactly)
const validNumericCases = [
  { sampleRate: 48000, hint: 1, expected: 1 }, // Absolute min bound
  { sampleRate: 48000, hint: 13, expected: 13 }, // Prime number < 128
  { sampleRate: 48000, hint: 127, expected: 127 }, // Just under 128
  { sampleRate: 48000, hint: 129, expected: 129 }, // Just over 128
  { sampleRate: 48000, hint: 256, expected: 256 }, // Power-of-two
  { sampleRate: 48000, hint: 500, expected: 500 }, // Arbitrary size
  { sampleRate: 48000, hint: 16383, expected: 16383 }, // Large size
  { sampleRate: 3000, hint: 18000, expected: 18000 }, // 6s limit @ 3kHz
  { sampleRate: 48000, hint: 288000, expected: 288000 }, // 6s limit @ 48kHz
  { sampleRate: 44100.1, hint: 264600, expected: 264600 }, // Floored 6s limit
];

// Comprehensive out-of-bounds combinations (should throw NotSupportedError)
const invalidNumericCases = [
  { sampleRate: 48000, hint: 0 }, // Less than 1
  { sampleRate: 3000, hint: 18001 }, // Exceeds 6s limit (18000)
  { sampleRate: 48000, hint: 288001 }, // Exceeds 6s limit (288000)
  { sampleRate: 48000, hint: -1 }, // Negative overflow
];

function runRenderSizeHintTests(contextType) {
  const isOffline = contextType === 'OfflineAudioContext';

  // Keyword-based cases (no-hint, default, and hardware)
  keywordCases.forEach(({ hint, description }) => {
    test(function() {
      let context;
      const options = {};
      if (hint !== undefined) {
        options.renderSizeHint = hint;
      }

      if (isOffline) {
        options.length = OFFLINE_CONTEXT_LENGTH;
        options.sampleRate = 44100;
        context = new OfflineAudioContext(options);
      } else {
        context = new AudioContext(options);
      }

      if (hint === 'hardware' && !isOffline) {
        assert_greater_than(context.renderQuantumSize, 0,
            `renderQuantumSize with "hardware" hint`);
      } else {
        assert_equals(context.renderQuantumSize, 128,
            `renderQuantumSize with ${description} hint`);
      }

      if (!isOffline) {
        context.close();
      }
    }, `${contextType} with ${description} hint`);
  });

  // Numeric valid combinations (honored exactly)
  validNumericCases.forEach(({ sampleRate, hint, expected }) => {
    test(function() {
      let context;
      if (isOffline) {
        context = new OfflineAudioContext(
            {length: OFFLINE_CONTEXT_LENGTH, sampleRate, renderSizeHint: hint});
      } else {
        context = new AudioContext({ sampleRate, renderSizeHint: hint });
      }
      assert_equals(context.renderQuantumSize, expected,
          `renderQuantumSize should be exactly ${expected}`);
      if (!isOffline) {
        context.close();
      }
    }, `${contextType} [Honored Exactly]: sampleRate ${sampleRate}, ` +
       `renderSizeHint ${hint}`);
  });

  // Out-of-bounds combinations (should throw NotSupportedError)
  invalidNumericCases.forEach(({ sampleRate, hint }) => {
    test(function() {
      assert_throws_dom('NotSupportedError', () => {
        if (isOffline) {
          new OfflineAudioContext(
              {
                length: OFFLINE_CONTEXT_LENGTH,
                sampleRate,
                renderSizeHint: hint
              });
        } else {
          new AudioContext({ sampleRate, renderSizeHint: hint });
        }
      });
    }, `${contextType} [Throws Exception]: sampleRate ${sampleRate}, ` +
       `renderSizeHint ${hint}`);
  });
}
