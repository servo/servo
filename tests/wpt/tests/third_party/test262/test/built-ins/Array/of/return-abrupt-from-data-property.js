// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.of
description: >
  Return abrupt from Data Property creation
info: |
  Array.of ( ...items )

  ...
  7. Let k be 0.
  8. Repeat, while k < len
    a. Let kValue be items[k].
    b. Let Pk be ToString(k).
    c. Let defineStatus be CreateDataPropertyOrThrow(A,Pk, kValue).
    d. ReturnIfAbrupt(defineStatus).
  ...

  7.3.6 CreateDataPropertyOrThrow (O, P, V)

  ...
  3. Let success be CreateDataProperty(O, P, V).
  4. ReturnIfAbrupt(success).
  5. If success is false, throw a TypeError exception.
  ...
---*/

function T1() {
  Object.preventExtensions(this);
}

assert.throws(TypeError, function() {
  Array.of.call(T1, 'Bob');
}, 'Array.of.call(T1, "Bob") throws a TypeError exception');

function T2() {
  Object.defineProperty(this, 0, {
    configurable: false,
    writable: true,
    enumerable: true
  });
}

assert.throws(TypeError, function() {
  Array.of.call(T2, 'Bob');
}, 'Array.of.call(T2, "Bob") throws a TypeError exception')
