// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Infinity is ReadOnly
es5id: 15.1.1.2_A2_T2
description: Checking typeof Functions
flags: [noStrict]
---*/

// CHECK#1
Infinity = true;
assert.notSameValue(typeof(Infinity), "boolean", 'The value of typeof(Infinity) is not "boolean"');

// TODO: Convert to verifyProperty() format.
