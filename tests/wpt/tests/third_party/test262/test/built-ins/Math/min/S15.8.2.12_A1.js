// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If no arguments are given, Math.min() is +Infinity
es5id: 15.8.2.12_A1
description: Checking if Math.min() equals to +Infinity
---*/
assert.sameValue(Math.min(), +Infinity, 'Math.min() must return +Infinity');
