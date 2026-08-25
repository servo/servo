// Copyright (C) 2020 Sony Interactive Entertainment Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.unshift
description: >
  Array#unshift throws TypeError upon attempting to modify a string
info: |
  Array.prototype.unshift ( ...items )
  ...
  4. If argCount > 0, then
    ...
    c. Repeat, while k > 0,
      ...
      iv. If fromPresent is true, then
        ...
        2. Perform ? Set(O, to, fromValue, true).
      ...
    ...
    f. Repeat, while items is not empty
      ...
      Perform ? Set(O, ! ToString(j), E, true).
      ...
  5. Perform ? Set(O, "length", len + argCount, true).

  Set ( O, P, V, Throw )
  ...
  4. Let success be ? O.[[Set]](P, V, O).
  5. If success is false and Throw is true, throw a TypeError exception.
---*/

assert.throws(TypeError, () => {
  Array.prototype.unshift.call('');
}, "Array.prototype.unshift.call('')");

assert.throws(TypeError, () => {
  Array.prototype.unshift.call('', 1);
}, "Array.prototype.unshift.call('', 1)");

assert.throws(TypeError, () => {
  Array.prototype.unshift.call('abc');
}, "Array.prototype.unshift.call('abc')");

assert.throws(TypeError, () => {
  Array.prototype.unshift.call('abc', 1);
}, "Array.prototype.unshift.call('abc', 1)");
