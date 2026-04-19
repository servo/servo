// Copyright (C) 2021 Jamie Kyle.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.hasown
description: Object.hasOwn called with symbol property key
info: |
  Object.hasOwn ( _O_, _P_ )

  1. Let _obj_ be ? ToObject(_O_).
  1. Let _key_ be ? ToPropertyKey(_P_).
  ...
author: Jamie Kyle
features: [Symbol, Object.hasOwn]
---*/

var obj = {};
var sym = Symbol();

assert.sameValue(
  Object.hasOwn(obj, sym),
  false,
  "Returns false if symbol own property not found"
);

obj[sym] = 0;

assert.sameValue(
  Object.hasOwn(obj, sym),
  true,
  "Returns true if symbol own property found"
);
