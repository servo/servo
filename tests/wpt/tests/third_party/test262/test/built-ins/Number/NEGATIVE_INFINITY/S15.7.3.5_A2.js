// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number.NEGATIVE_INFINITY is ReadOnly
es5id: 15.7.3.5_A2
description: Checking if varying Number.NEGATIVE_INFINITY fails
includes: [propertyHelper.js]
---*/

// CHECK#1
verifyNotWritable(Number, "NEGATIVE_INFINITY", null, 1);

assert(
  !isFinite(Number.NEGATIVE_INFINITY),
  'The value of !isFinite(Number.NEGATIVE_INFINITY) is expected to be true'
);

// TODO: Convert to verifyProperty() format.
