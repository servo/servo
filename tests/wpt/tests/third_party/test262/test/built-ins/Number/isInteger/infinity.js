// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 20.1.2.3
author: Ryan Lewis
description: Number.isInteger should return false if called with Infinity.
---*/

assert.sameValue(Number.isInteger(Infinity), false, 'Infinity');
assert.sameValue(Number.isInteger(-Infinity), false, '-Infinity');
