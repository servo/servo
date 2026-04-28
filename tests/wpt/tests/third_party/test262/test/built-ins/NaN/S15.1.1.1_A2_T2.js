// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The NaN is ReadOnly
es5id: 15.1.1.1_A2_T2
description: Checking typeof Functions
flags: [noStrict]
---*/

// CHECK#1
NaN = true;
assert.notSameValue(typeof(NaN), "boolean", 'The value of typeof(NaN) is not "boolean"');

// TODO: Convert to verifyProperty() format.
