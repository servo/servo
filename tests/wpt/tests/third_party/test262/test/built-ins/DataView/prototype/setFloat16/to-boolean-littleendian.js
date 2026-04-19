// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: >
  Boolean littleEndian argument coerced in ToBoolean
features: [Float16Array, Symbol]
---*/

var buffer = new ArrayBuffer(2);
var sample = new DataView(buffer, 0);

// False
sample.setFloat16(0, 1);
assert.sameValue(sample.getFloat16(0), 1, "no arg");
sample.setFloat16(0, 2, undefined);
assert.sameValue(sample.getFloat16(0), 2, "undefined");
sample.setFloat16(0, 3, null);
assert.sameValue(sample.getFloat16(0), 3, "null");
sample.setFloat16(0, 4, 0);
assert.sameValue(sample.getFloat16(0), 4, "0");
sample.setFloat16(0, 5, "");
assert.sameValue(sample.getFloat16(0), 5, "the empty string");

// True
sample.setFloat16(0, 6, {}); // 01000110 00000000
assert.sameValue(sample.getFloat16(0), 0.000004172325134277344, "{}"); // 00000000 01000110
sample.setFloat16(0, 7, Symbol("1")); // 01000111 00000000
assert.sameValue(sample.getFloat16(0), 0.000004231929779052734, "symbol"); // 00000000 01000111
sample.setFloat16(0, 8, 1); // 01001000 00000000
assert.sameValue(sample.getFloat16(0), 0.000004291534423828125, "1"); // 00000000 01001000
sample.setFloat16(0, 9, "string"); // 01001000 10000000
assert.sameValue(sample.getFloat16(0), -0.000004291534423828125, "string"); // 10000000 01001000
