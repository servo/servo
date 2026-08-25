// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If no arguments are given, Math.max() is -Infinity
es5id: 15.8.2.11_A1
description: Checking if Math.max() equals to -Infinity
---*/
assert.sameValue(Math.max(), -Infinity, 'Math.max() must return -Infinity');
