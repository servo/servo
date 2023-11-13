// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js

'use strict';

test(() => {
  const context = new OfflineAudioContext(1, 1, 44100);
  const defaultValue = -1;
  const gainNode = new GainNode(context, { gain: defaultValue });

  assert_equals(gainNode.gain.defaultValue, defaultValue, "AudioParam's defaultValue is not correct.");
}, "AudioParam's defaultValue");
