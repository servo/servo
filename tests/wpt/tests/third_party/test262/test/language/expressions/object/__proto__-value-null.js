// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-__proto__-property-names-in-object-initializers
es6id: B.3.1
description: >
  The value of the `__proto__` property key is assigned to the [[Prototype]]
  internal slot (null value)
info: |
  ...
  6. If propKey is the String value "__proto__" and if
     IsComputedPropertyKey(propKey) is false, then
     a. If Type(propValue) is either Object or Null, then
        i. Return object.[[SetPrototypeOf]](propValue).
---*/

var object = {
  __proto__: null
};

assert.sameValue(Object.getPrototypeOf(object), null);
assert.sameValue(
  Object.getOwnPropertyDescriptor(object, '__proto__'), undefined
);
