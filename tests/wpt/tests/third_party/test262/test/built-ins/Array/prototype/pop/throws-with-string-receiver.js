// Copyright (C) 2020 Sony Interactive Entertainment Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.pop
description: >
  Array#pop throws TypeError upon attempting to modify a string
info: |
  Array.prototype.pop ( )
  ...
  3. If len is zero, then
    a. Perform ? Set(O, "length", 0, true).
    ...
  4. Else,
    ...
    f. Perform ? Set(O, "length", newLen, true).

  Set ( O, P, V, Throw )
  ...
  4. Let success be ? O.[[Set]](P, V, O).
  5. If success is false and Throw is true, throw a TypeError exception.
---*/

assert.throws(TypeError, () => {
  Array.prototype.pop.call('');
}, "Array.prototype.pop.call('')");

assert.throws(TypeError, () => {
  Array.prototype.pop.call('abc');
}, "Array.prototype.pop.call('abc')");
