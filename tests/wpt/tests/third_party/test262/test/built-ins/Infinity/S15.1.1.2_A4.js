// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Infinity is DontEnum
es5id: 15.1.1.2_A4
description: Use for-in statement
---*/

// CHECK#1
for (var prop in this) {
  assert.notSameValue(prop, "Infinity", 'The value of prop is not "Infinity"');
}

// TODO: Convert to verifyProperty() format.
