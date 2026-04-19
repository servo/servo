// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The NaN is DontEnum
es5id: 15.1.1.1_A3.2
description: Use for-in statement
---*/

// CHECK#1
for (var prop in this) {
  assert.notSameValue(prop, "NaN", 'The value of prop is not "NaN"');
}

// TODO: Convert to verifyProperty() format.
