// Copyright (C) 2023 Anthony Frehner. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-getsetrecord
description: GetSetRecord throws if obj is not an object
info: |
    1. If obj is not an Object, throw a TypeError exception.
features: [set-methods]
---*/

let s1 = new Set([1]);
assert.throws(
  TypeError,
  function () {
    s1.union(1);
  },
  "number"
);

assert.throws(
  TypeError,
  function () {
    s1.union("");
  },
  "string"
);

assert.throws(
  TypeError,
  function () {
    s1.union(1n);
  },
  "bigint"
);

assert.throws(
  TypeError,
  function () {
    s1.union(false);
  },
  "boolean"
);

assert.throws(
  TypeError,
  function () {
    s1.union(undefined);
  },
  "undefined"
);

assert.throws(
  TypeError,
  function () {
    s1.union(null);
  },
  "null"
);

assert.throws(
  TypeError,
  function () {
    s1.union(Symbol("test"));
  },
  "symbol"
);
