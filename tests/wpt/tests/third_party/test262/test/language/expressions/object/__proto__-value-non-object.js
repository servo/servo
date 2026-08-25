// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-__proto__-property-names-in-object-initializers
es6id: B.3.1
description: >
  The value of the `__proto__` property key is not assigned to the
  [[Prototype]] internal slot, nor to a property named "__proto__" (non-Object,
  non-null value)
info: |
  ...
  6. If propKey is the String value "__proto__" and if
     IsComputedPropertyKey(propKey) is false, then
     a. If Type(propValue) is either Object or Null, then
        [...]
     b. Return NormalCompletion(empty).
features: [Symbol]
---*/

var object;

object = {
  __proto__: undefined
};
assert.sameValue(
  Object.getPrototypeOf(object),
  Object.prototype,
  'prototype (undefined)'
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(object, '__proto__'),
  undefined,
  'property (undefined)'
);

object = {
  __proto__: 1
};
assert.sameValue(
  Object.getPrototypeOf(object),
  Object.prototype,
  'prototype (numeric primitive)'
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(object, '__proto__'),
  undefined,
  'property (numeric primitive)'
);

object = {
  __proto__: false
};
assert.sameValue(
  Object.getPrototypeOf(object),
  Object.prototype,
  'prototype (boolean primitive)'
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(object, '__proto__'),
  undefined,
  'property (boolean primitive)'
);

object = {
  __proto__: 'string literal'
};
assert.sameValue(
  Object.getPrototypeOf(object),
  Object.prototype,
  'prototype (string primitive)'
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(object, '__proto__'),
  undefined,
  'property (string primitive)'
);

object = {
  __proto__: Symbol('')
};
assert.sameValue(
  Object.getPrototypeOf(object),
  Object.prototype,
  'prototype (symbol)'
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(object, '__proto__'),
  undefined,
  'property (symbol)'
);
