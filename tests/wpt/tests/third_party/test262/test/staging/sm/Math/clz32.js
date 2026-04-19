// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Undefined and NaN end up as zero after ToUint32
assert.sameValue(Math.clz32(), 32);
assert.sameValue(Math.clz32(NaN), 32);
assert.sameValue(Math.clz32.call(), 32);
// 0
assert.sameValue(Math.clz32(null), 32);
assert.sameValue(Math.clz32(false), 32);
// 1
assert.sameValue(Math.clz32(true), 31);
// 3
assert.sameValue(Math.clz32(3.5), 30);
// NaN -> 0
assert.sameValue(Math.clz32({}), 32);
// 2
assert.sameValue(Math.clz32({valueOf: function() { return 2; }}), 30);
// 0 -> 0
assert.sameValue(Math.clz32([]), 32);
assert.sameValue(Math.clz32(""), 32);
// NaN -> 0
assert.sameValue(Math.clz32([1, 2, 3]), 32);
assert.sameValue(Math.clz32("bar"), 32);
// 15
assert.sameValue(Math.clz32("15"), 28);


assert.sameValue(Math.clz32(0x80000000), 0);
assert.sameValue(Math.clz32(0xF0FF1000), 0);
assert.sameValue(Math.clz32(0x7F8F0001), 1);
assert.sameValue(Math.clz32(0x3FFF0100), 2);
assert.sameValue(Math.clz32(0x1FF50010), 3);
assert.sameValue(Math.clz32(0x00800000), 8);
assert.sameValue(Math.clz32(0x00400000), 9);
assert.sameValue(Math.clz32(0x00008000), 16);
assert.sameValue(Math.clz32(0x00004000), 17);
assert.sameValue(Math.clz32(0x00000080), 24);
assert.sameValue(Math.clz32(0x00000040), 25);
assert.sameValue(Math.clz32(0x00000001), 31);
assert.sameValue(Math.clz32(0), 32);

