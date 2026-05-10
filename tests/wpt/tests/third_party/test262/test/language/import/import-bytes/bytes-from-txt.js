// Copyright (C) 2025 @styfle. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-create-bytes-module
description: Creates bytes module from txt file
flags: [module]
features: [import-attributes, immutable-arraybuffer, import-bytes]
includes: [compareArray.js]
---*/

import value from './bytes-from-txt_FIXTURE.txt' with { type: 'bytes' };

assert(value instanceof Uint8Array);
assert(value.buffer instanceof ArrayBuffer);

assert.sameValue(value.length, 13);
assert.sameValue(value.buffer.byteLength, 13);
assert.sameValue(value.buffer.immutable, true);

assert.compareArray(
  Array.from(value),
  [
    0x48, // H
    0x65, // e
    0x6c, // l
    0x6c, // l
    0x6f, // o
    0x20, // (space)
    0x57, // W
    0x6f, // o
    0x72, // r
    0x6c, // l
    0x64, // d
    0x21, // !
    0x0a, // (newline)
  ]
);

assert.throws(TypeError, function() {
  value.buffer.resize(0);
});

assert.throws(TypeError, function() {
  value.buffer.transfer();
});
