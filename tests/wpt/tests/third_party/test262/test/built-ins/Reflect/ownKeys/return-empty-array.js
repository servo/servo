// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.11
description: >
  Returns empty array when target has no own properties.
info: |
  26.1.11 Reflect.ownKeys ( target )

  ...
  2. Let keys be target.[[OwnPropertyKeys]]().
  3. ReturnIfAbrupt(keys).
  4. Return CreateArrayFromList(keys).
includes: [compareArray.js]
features: [Reflect]
---*/

assert.compareArray(Reflect.ownKeys({}), []);

var o = {
  d: 42
};
delete o.d;
assert.compareArray(Reflect.ownKeys(o), []);
