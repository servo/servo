// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.7
description: >
  Return a property descriptor object as an accessor descriptor.
info: |
  26.1.7 Reflect.getOwnPropertyDescriptor ( target, propertyKey )

  ...
  4. Let desc be target.[[GetOwnProperty]](key).
  5. ReturnIfAbrupt(desc).
  6. Return FromPropertyDescriptor(desc).

  6.2.4.4 FromPropertyDescriptor ( Desc )

  ...
  2. Let obj be ObjectCreate(%ObjectPrototype%).
  ...
  4. If Desc has a [[Value]] field, then
    a. Perform CreateDataProperty(obj, "value", Desc.[[Value]]).
  5. If Desc has a [[Writable]] field, then
    a. Perform CreateDataProperty(obj, "writable", Desc.[[Writable]]).
  6. If Desc has a [[Get]] field, then
    a. Perform CreateDataProperty(obj, "get", Desc.[[Get]]).
  7. If Desc has a [[Set]] field, then
    a. Perform CreateDataProperty(obj, "set", Desc.[[Set]])
  8. If Desc has an [[Enumerable]] field, then
    a. Perform CreateDataProperty(obj, "enumerable", Desc.[[Enumerable]]).
  9. If Desc has a [[Configurable]] field, then
    a. Perform CreateDataProperty(obj , "configurable", Desc.[[Configurable]]).
  ...
  11. Return obj.

includes: [compareArray.js]
features: [Reflect]
---*/

var o1 = {};
var fn = function() {};
Object.defineProperty(o1, 'p', {
  get: fn,
  configurable: true
});

var result = Reflect.getOwnPropertyDescriptor(o1, 'p');

assert.compareArray(
  Object.getOwnPropertyNames(result),
  ['get', 'set', 'enumerable', 'configurable']
);
assert.sameValue(result.enumerable, false);
assert.sameValue(result.configurable, true);
assert.sameValue(result.get, fn);
assert.sameValue(result.set, undefined);
