// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Ensure the original and expected values are set properly
includes: [byteConversionValues.js]
---*/

var values = byteConversionValues.values;
var expected = byteConversionValues.expected;

assert(values.length > 0);
assert.sameValue(values.length, expected.Float32.length, "Float32");
assert.sameValue(values.length, expected.Float64.length, "Float64");
assert.sameValue(values.length, expected.Int8.length, "Int8");
assert.sameValue(values.length, expected.Int16.length, "Int16");
assert.sameValue(values.length, expected.Int32.length, "Int32");
assert.sameValue(values.length, expected.Uint8.length, "Uint8");
assert.sameValue(values.length, expected.Uint16.length, "Uint16");
assert.sameValue(values.length, expected.Uint32.length, "Uint32");
assert.sameValue(values.length, expected.Uint8Clamped.length, "Uint8Clamped");
