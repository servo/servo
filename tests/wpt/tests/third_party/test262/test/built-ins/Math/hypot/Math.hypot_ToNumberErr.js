// Copyright (c) 2021 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Math.hypot should coerce all arguments before inspecting them.
esid: sec-math.hypot
info: |
  1. Let _coerced_ be a new empty List.
  2. For each element _arg_ of _args_, do
    a. Let _n_ be ? ToNumber(_arg_).
    b. Append _n_ to _coerced_.
  3. For each element _number_ of _coerced_, do
---*/

var counter = 0;

assert.throws(
  Test262Error,
  function() {
    Math.hypot(
      Infinity,
      -Infinity,
      NaN,
      0,
      -0,
      {valueOf: function(){ throw new Test262Error(); }},
      {valueOf: function(){ counter++; }}
    );
  },
  'Math.hypot propagates an abrupt completion from coercing an argument to Number'
);

assert.sameValue(counter, 0,
    'Math.hypot aborts argument processing at the first abrupt completion');
