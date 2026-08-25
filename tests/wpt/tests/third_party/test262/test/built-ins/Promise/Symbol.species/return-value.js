// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.4.6
description: Promise `Symbol.species` accessor function return value
info: |
    1. Return the this value.
features: [Symbol.species]
---*/

var desc = Object.getOwnPropertyDescriptor(Promise, Symbol.species);
var thisValue = {};

assert.sameValue(desc.get.call(thisValue), thisValue);
