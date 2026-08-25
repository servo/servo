// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.charat
description: Coercion of "pos" string value into number
info: |
  [...]
  3. Let position be ? ToInteger(pos).
  [...]

  7.1.4 ToInteger

  1. Let number be ? ToNumber(argument).
---*/

assert.sameValue('abcd'.charAt('   +00200.0000E-0002   '), 'c');
