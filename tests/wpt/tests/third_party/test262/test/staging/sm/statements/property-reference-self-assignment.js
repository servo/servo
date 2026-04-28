// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var hits = 0;

var p = { toString() { hits++; return "prop" } };
var obj = { foo: 1 };


var ops = [["obj[p]++", 2],
           ["++obj[p]", 2],
           ["--obj[p]", 0],
           ["obj[p]--", 0],
           ["obj[p] += 2", 3],
           ["obj[p] -= 2", -1],
           ["obj[p] *= 2", 2],
           ["obj[p] /= 2", 0.5],
           ["obj[p] %= 2", 1],
           ["obj[p] >>>= 2", 0],
           ["obj[p] >>= 2", 0],
           ["obj[p] <<= 2", 4],
           ["obj[p] |= 2", 3],
           ["obj[p] ^= 2", 3],
           ["obj[p] &= 2", 0]];

var testHits = 0;
for (let op of ops) {
    // Seed the value for each test.
    obj.prop = 1;

    // Do the operation.
    eval(op[0]);
    assert.sameValue(obj.prop, op[1], `value for ${op[0]}`);

    // We should always call toString once, for each operation.
    testHits++;
    assert.sameValue(hits, testHits, `toString calls for ${op[0]}`);
}

