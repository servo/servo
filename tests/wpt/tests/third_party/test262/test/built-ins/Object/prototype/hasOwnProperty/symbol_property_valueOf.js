// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.prototype.hasOwnProperty with symbol and valueOf conversion
info: |
  19.1.3.2 Object.prototype.hasOwnProperty ( V )

  1. Let P be ToPropertyKey(V).
  2. ReturnIfAbrupt(P).
  ...
es6id: 19.1.3.2
features: [Symbol]
---*/

var obj = {};
var sym = Symbol();

var callCount = 0;
var wrapper = {
  valueOf: function() {
    callCount += 1;
    return sym;
  },
  toString: null
};

obj[sym] = 0;

assert.sameValue(
  obj.hasOwnProperty(wrapper),
  true,
  "Returns true if symbol own property found"
);

assert.sameValue(callCount, 1, "valueOf method called exactly once");
