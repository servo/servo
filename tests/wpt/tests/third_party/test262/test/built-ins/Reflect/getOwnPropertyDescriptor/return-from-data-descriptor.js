// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.7
description: >
  Return a property descriptor object as a data descriptor.
info: |
  26.1.7 Reflect.getOwnPropertyDescriptor ( target, propertyKey )

  ...
  4. Let desc be target.[[GetOwnProperty]](key).
  5. ReturnIfAbrupt(desc).
  6. Return FromPropertyDescriptor(desc).
includes: [compareArray.js]
features: [Reflect]
---*/

var o1 = {
  p: 'foo'
};

var result = Reflect.getOwnPropertyDescriptor(o1, 'p');

assert.compareArray(
  Object.getOwnPropertyNames(result),
  ['value', 'writable', 'enumerable', 'configurable']
);
assert.sameValue(result.value, 'foo');
assert.sameValue(result.enumerable, true);
assert.sameValue(result.configurable, true);
assert.sameValue(result.writable, true);
