// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.f16round
description: >
  Convert to binary16 format and than to binary64 format
features: [Float16Array]
includes: [byteConversionValues.js]
---*/

var values = byteConversionValues.values;
var expectedValues = byteConversionValues.expected.Float16;

values.forEach(function(value, i) {
  var expected = expectedValues[i];

  var result = Math.f16round(value);

  assert.sameValue(
    result,
    expected,
    "value: " + value
  );
});
