// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-getsetrecord
description: GetSetRecord throws an exception if the Set-like object has a size that is coerced to NaN
info: |
    2. Let rawSize be ? Get(obj, "size").
    3. Let numSize be ? ToNumber(rawSize).
    4. NOTE: If rawSize is undefined, then numSize will be NaN.
    5. If numSize is NaN, throw a TypeError exception.
features: [set-methods]
---*/

const s1 = new Set([1, 2]);
const s2 = {
  size: undefined,
  has: () => {},
  keys: function* keys() {
    yield 2;
    yield 3;
  },
};
assert.throws(
  TypeError,
  function () {
    s1.difference(s2);
  },
  "GetSetRecord throws an error when size is undefined"
);

s2.size = NaN;
assert.throws(
  TypeError,
  function () {
    s1.difference(s2);
  },
  "GetSetRecord throws an error when size is NaN"
);

let coercionCalls = 0;
s2.size = {
  valueOf: function() {
    ++coercionCalls;
    return NaN;
  },
};
assert.throws(
  TypeError,
  function () {
    s1.difference(s2);
  },
  "GetSetRecord throws an error when size coerces to NaN"
);
assert.sameValue(coercionCalls, 1, "GetSetRecord coerces size");

s2.size = 0n;
assert.throws(
  TypeError,
  function () {
    s1.difference(s2);
  },
  "GetSetRecord throws an error when size is a BigInt"
);

s2.size = "string";
assert.throws(
  TypeError,
  function () {
    s1.difference(s2);
  },
  "GetSetRecord throws an error when size is a non-numeric string"
);
