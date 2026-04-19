// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-promise-prototype-object
description: Promise.prototype does not have a [[PromiseState]] internal slot
info: |
  The Promise prototype object is the intrinsic object %PromisePrototype%. The
  value of the [[Prototype]] internal slot of the Promise prototype object is
  the intrinsic object %ObjectPrototype%. The Promise prototype object is an
  ordinary object. It does not have a [[PromiseState]] internal slot or any of
  the other internal slots of Promise instances.

  25.4.5.3 Promise.prototype.then

  1. Let promise be the this value.
  2. If IsPromise(promise) is false, throw a TypeError exception.

  25.4.1.6 IsPromise

  1. If Type(x) is not Object, return false.
  2. If x does not have a [[PromiseState]] internal slot, return false.
---*/

assert.throws(TypeError, function() {
  Promise.prototype.then.call(Promise.prototype, function() {}, function() {});
});
