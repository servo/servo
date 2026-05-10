// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
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
    s1.difference(1);
  },
  "number"
);

assert.throws(
  TypeError,
  function () {
    s1.difference("");
  },
  "string"
);

assert.throws(
  TypeError,
  function () {
    s1.difference(1n);
  },
  "bigint"
);

assert.throws(
  TypeError,
  function () {
    s1.difference(false);
  },
  "boolean"
);

assert.throws(
  TypeError,
  function () {
    s1.difference(undefined);
  },
  "undefined"
);

assert.throws(
  TypeError,
  function () {
    s1.difference(null);
  },
  "null"
);

assert.throws(
  TypeError,
  function () {
    s1.difference(Symbol("test"));
  },
  "symbol"
);
