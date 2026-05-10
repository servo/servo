// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.10
description: >
  Returns the boolean result.
info: |
  26.1.10 Reflect.isExtensible (target)

  ...
  2. Return target.[[IsExtensible]]().
features: [Reflect]
---*/

var o = {};
assert.sameValue(Reflect.isExtensible(o), true);

Object.preventExtensions(o);
assert.sameValue(Reflect.isExtensible(o), false);
