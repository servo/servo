// Copyright (C) 2025 @styfle. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-create-bytes-module
description: Creates bytes module from png file
flags: [module]
features: [import-attributes, immutable-arraybuffer, import-bytes]
includes: [compareArray.js]
---*/

import value from './bytes-from-png_FIXTURE.png' with { type: 'bytes' };

assert(value instanceof Uint8Array);
assert(value.buffer instanceof ArrayBuffer);

assert.sameValue(value.length, 67);
assert.sameValue(value.buffer.byteLength, 67);
assert.sameValue(value.buffer.immutable, true);

assert.compareArray(
  Array.from(value),
  [
    0x89,
    0x50,
    0x4e,
    0x47,
    0xd,
    0xa,
    0x1a,
    0xa,
    0x0,
    0x0,
    0x0,
    0xd,
    0x49,
    0x48,
    0x44,
    0x52,
    0x0,
    0x0,
    0x0,
    0x1,
    0x0,
    0x0,
    0x0,
    0x1,
    0x1,
    0x0,
    0x0,
    0x0,
    0x0,
    0x37,
    0x6e,
    0xf9,
    0x24,
    0x0,
    0x0,
    0x0,
    0xa,
    0x49,
    0x44,
    0x41,
    0x54,
    0x78,
    0x1,
    0x63,
    0x60,
    0x0,
    0x0,
    0x0,
    0x2,
    0x0,
    0x1,
    0x73,
    0x75,
    0x1,
    0x18,
    0x0,
    0x0,
    0x0,
    0x0,
    0x49,
    0x45,
    0x4e,
    0x44,
    0xae,
    0x42,
    0x60,
    0x82,
  ]
);

assert.throws(TypeError, function() {
  value.buffer.resize(0);
});

assert.throws(TypeError, function() {
  value.buffer.transfer();
});
