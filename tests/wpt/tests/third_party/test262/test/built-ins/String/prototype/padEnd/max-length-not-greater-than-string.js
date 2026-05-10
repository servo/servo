// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.padend
description: >
  String#padEnd should return the string unchanged when an integer max
  length is not greater than the string length
author: Jordan Harband
---*/

assert.sameValue('abc'.padEnd(undefined, 'def'), 'abc');
assert.sameValue('abc'.padEnd(null, 'def'), 'abc');
assert.sameValue('abc'.padEnd(NaN, 'def'), 'abc');
assert.sameValue('abc'.padEnd(-Infinity, 'def'), 'abc');
assert.sameValue('abc'.padEnd(0, 'def'), 'abc');
assert.sameValue('abc'.padEnd(-1, 'def'), 'abc');
assert.sameValue('abc'.padEnd(3, 'def'), 'abc');
assert.sameValue('abc'.padEnd(3.9999, 'def'), 'abc');
