// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: A matching element of indices is an Array with exactly two number properties.
esid: sec-getmatchindicesarray
features: [regexp-match-indices]
info: |
  GetMatchIndicesArray ( S, match )
    5. Return CreateArrayFromList(« _match_.[[StartIndex]], _match_.[[EndIndex]] »).
---*/

let input = "abcd";
let match = /b(c)/d.exec(input);
let indices = match.indices;

// `indices[0]` is an array
assert.sameValue(Object.getPrototypeOf(indices[0]), Array.prototype);
assert.sameValue(indices[0].length, 2);
assert.sameValue(typeof indices[0][0], "number");
assert.sameValue(typeof indices[0][1], "number");

// `indices[1]` is an array
assert.sameValue(Object.getPrototypeOf(indices[1]), Array.prototype);
assert.sameValue(indices[1].length, 2);
assert.sameValue(typeof indices[1][0], "number");
assert.sameValue(typeof indices[1][1], "number");
