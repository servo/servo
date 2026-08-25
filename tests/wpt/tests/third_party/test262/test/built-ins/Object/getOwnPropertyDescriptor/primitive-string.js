// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.getownpropertydescriptor
description: >
  String primitive as first argument is coerced to object.
info: |
  Object.getOwnPropertyDescriptor ( O, P )

  1. Let obj be ? ToObject(O).
  [...]
  3. Let desc be ? obj.[[GetOwnProperty]](key).
  4. Return FromPropertyDescriptor(desc).

  String Exotic Objects

  String exotic objects always have a data property named "length" whose value is the number
  of code unit elements in the encapsulated String value. Both the code unit data properties
  and the "length" property are non-writable and non-configurable.
---*/

assert.sameValue(Object.getOwnPropertyDescriptor('', '0'), undefined);

var indexDesc = Object.getOwnPropertyDescriptor('foo', '0');

assert.sameValue(indexDesc.value, 'f', '[[Value]]');
assert.sameValue(indexDesc.writable, false, '[[Writable]]');
assert.sameValue(indexDesc.enumerable, true, '[[Enumerable]]');
assert.sameValue(indexDesc.configurable, false, '[[Configurable]]');

var lengthDesc = Object.getOwnPropertyDescriptor('foo', 'length');

assert.sameValue(lengthDesc.value, 3, '[[Value]]');
assert.sameValue(lengthDesc.writable, false, '[[Writable]]');
assert.sameValue(lengthDesc.enumerable, false, '[[Enumerable]]');
assert.sameValue(lengthDesc.configurable, false, '[[Configurable]]');
