// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.8
description: >
  Return null prototype.
info: |
  26.1.8 Reflect.getPrototypeOf ( target )

  ...
  2. Return target.[[GetPrototypeOf]]().
features: [Reflect]
---*/

var o = Object.create(null);
assert.sameValue(Reflect.getPrototypeOf(o), null);
