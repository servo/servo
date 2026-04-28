// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-getsetrecord
description: GetSetRecord throws an exception if the Set-like object's 'has' property is not callable
info: |
    7. Let has be ? Get(obj, "has").
    8. If IsCallable(has) is false, throw a TypeError exception.
features: [set-methods]
---*/

const s1 = new Set([1, 2]);
const s2 = {
  size: 2,
  has: undefined,
  keys: function* keys() {
    yield 2;
    yield 3;
  },
};
assert.throws(
  TypeError,
  function () {
    s1.isSubsetOf(s2);
  },
  "GetSetRecord throws an error when has is undefined"
);

s2.has = {};
assert.throws(
  TypeError,
  function () {
    s1.isSubsetOf(s2);
  },
  "GetSetRecord throws an error when has is not callable"
);
