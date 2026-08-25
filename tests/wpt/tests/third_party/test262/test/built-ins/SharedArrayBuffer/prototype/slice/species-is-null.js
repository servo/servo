// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Uses default constructor is species constructor is null.
info: |
  SharedArrayBuffer.prototype.slice ( start, end )

features: [SharedArrayBuffer, Symbol.species]
---*/

var speciesConstructor = {};
speciesConstructor[Symbol.species] = null;

var arrayBuffer = new SharedArrayBuffer(8);
arrayBuffer.constructor = speciesConstructor;

var result = arrayBuffer.slice();
assert.sameValue(Object.getPrototypeOf(result), SharedArrayBuffer.prototype);
