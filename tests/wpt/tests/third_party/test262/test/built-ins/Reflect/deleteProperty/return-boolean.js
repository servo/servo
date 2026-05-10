// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.4
description: >
  Return boolean result.
info: |
  26.1.4 Reflect.deleteProperty ( target, propertyKey )

  ...
  4. Return target.[[Delete]](key).
features: [Reflect]
---*/

var o = {};

o.p1 = 'foo';
assert.sameValue(Reflect.deleteProperty(o, 'p1'), true);
assert.sameValue(o.hasOwnProperty('p1'), false);

o.p2 = 'foo';
Object.freeze(o);
assert.sameValue(Reflect.deleteProperty(o, 'p2'), false);
assert.sameValue(o.hasOwnProperty('p2'), true);
