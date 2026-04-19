// Copyright (C) 2021 Jamie Kyle.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.hasown
description: Object.hasOwn with symbol and valueOf conversion
info: |
  Object.hasOwn ( _O_, _P_ )

  1. Let _obj_ be ? ToObject(_O_).
  2. Let _key_ be ? ToPropertyKey(_P_).
  ...
author: Jamie Kyle
features: [Symbol, Object.hasOwn]
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
  Object.hasOwn(obj, wrapper),
  true,
  "Returns true if symbol own property found"
);

assert.sameValue(callCount, 1, "valueOf method called exactly once");
