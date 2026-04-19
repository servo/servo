// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    The prototype of generator functions declared as methods is the
    Generator Prototype.
es6id: 14.4.13
features: [generators]
---*/

var obj = { *method() {} };
assert.sameValue(
  Object.getPrototypeOf(obj.method),
  Object.getPrototypeOf(function*() {})
);
