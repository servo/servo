// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The undefined is DontEnum
esid: sec-undefined
description: Use for-in statement
---*/

// CHECK#1
for (var prop in this) {
  if (prop === "undefined") {
    throw new Test262Error('#1: The undefined is DontEnum');
  }
}
