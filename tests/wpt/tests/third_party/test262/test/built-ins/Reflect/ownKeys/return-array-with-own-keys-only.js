// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.11
description: >
  Returns target's own property keys only, ignore prototype keys.
info: |
  26.1.11 Reflect.ownKeys ( target )

  ...
  2. Let keys be target.[[OwnPropertyKeys]]().
  3. ReturnIfAbrupt(keys).
  4. Return CreateArrayFromList(keys).
includes: [compareArray.js]
features: [Reflect]
---*/

var proto = {
  foo: 1
};

var o = Object.create(proto);
o.p1 = 42;
o.p2 = 43;
o.p3 = 44;
assert.compareArray(Reflect.ownKeys(o), ['p1', 'p2', 'p3'], 'return object own keys');
