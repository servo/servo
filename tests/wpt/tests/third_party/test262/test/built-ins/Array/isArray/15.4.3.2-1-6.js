// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.isarray
es5id: 15.4.3.2-1-6
description: Array.isArray applied to String object
---*/

assert.sameValue(Array.isArray(new String("hello\nworld\\!")), false, 'Array.isArray(new String("hello\\nworld\\\\!")) must return false');
