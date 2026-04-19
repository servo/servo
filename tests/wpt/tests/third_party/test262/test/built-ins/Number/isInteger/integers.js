// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 20.1.2.3
author: Ryan Lewis
description: Number.isInteger should return true if called with an integer.
---*/

assert.sameValue(Number.isInteger(478), true, 'Number.isInteger(478)');
assert.sameValue(Number.isInteger(-0), true, '-0');
assert.sameValue(Number.isInteger(0), true, '0');
assert.sameValue(Number.isInteger(-1), true, '-1');
assert.sameValue(Number.isInteger(9007199254740991), true, '9007199254740991');
assert.sameValue(Number.isInteger(-9007199254740991), true, '-9007199254740991');
assert.sameValue(Number.isInteger(9007199254740992), true, '9007199254740992');
assert.sameValue(Number.isInteger(-9007199254740992), true, '-9007199254740992');
