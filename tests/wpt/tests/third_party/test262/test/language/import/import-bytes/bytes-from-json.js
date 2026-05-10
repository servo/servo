// Copyright (C) 2025 @styfle. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-create-bytes-module
description: Creates bytes module from json file
flags: [module]
features: [import-attributes, immutable-arraybuffer, import-bytes]
includes: [compareArray.js]
---*/

import value from './bytes-from-json_FIXTURE.json' with { type: 'bytes' };

assert(value instanceof Uint8Array);
assert(value.buffer instanceof ArrayBuffer);

assert.sameValue(value.length, 12);
assert.sameValue(value.buffer.byteLength, 12);
assert.sameValue(value.buffer.immutable, true);

assert.compareArray(
  Array.from(value),
  [
    0x7b, // {
    0x20, // (space)
    0x22, // "
    0x61, // a
    0x22, // "
    0x3a, // :
    0x20, // (space)
    0x34, // 4
    0x32, // 2
    0x20, // (space)
    0x7d, // }
    0x0a, // (newline)
  ]
);

assert.throws(TypeError, function() {
  value.buffer.resize(0);
});

assert.throws(TypeError, function() {
  value.buffer.transfer();
});
