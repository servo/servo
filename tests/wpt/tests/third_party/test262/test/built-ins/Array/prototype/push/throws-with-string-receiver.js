// Copyright (C) 2020 Sony Interactive Entertainment Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.push
description: >
  Array#push throws TypeError upon attempting to modify a string
info: |
  Array.prototype.push ( ...items )
  ...
  6. Repeat, while items is not empty
    ...
    b. Perform ? Set(O, ! ToString(len), E, true).
    ...
  7. Perform ? Set(O, "length", len, true).

  Set ( O, P, V, Throw )
  ...
  4. Let success be ? O.[[Set]](P, V, O).
  5. If success is false and Throw is true, throw a TypeError exception.
---*/

assert.throws(TypeError, () => {
  Array.prototype.push.call('');
}, "Array.prototype.push.call('')");

assert.throws(TypeError, () => {
  Array.prototype.push.call('', 1);
}, "Array.prototype.push.call('', 1)");

assert.throws(TypeError, () => {
  Array.prototype.push.call('abc');
}, "Array.prototype.push.call('abc')");

assert.throws(TypeError, () => {
  Array.prototype.push.call('abc', 1);
}, "Array.prototype.push.call('abc', 1)");
