// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-__proto__-property-names-in-object-initializers
description: >
  The value of the `__proto__` property key is assigned to the [[Prototype]].
  Object.prototype.__proto__ setter should not be called.
info: |
  __proto__ Property Names in Object Initializers

  PropertyDefinition : PropertyName : AssignmentExpression

  [...]
  7. If isProtoSetter is true, then
    a. If Type(propValue) is either Object or Null, then
      i. Return object.[[SetPrototypeOf]](propValue).
---*/

Object.defineProperty(Object.prototype, '__proto__', {
  set: function() {
    throw new Test262Error('should not be called');
  },
});

var proto = {};

var object = {
  __proto__: proto
};

assert(!object.hasOwnProperty('__proto__'));
assert.sameValue(Object.getPrototypeOf(object), proto);
