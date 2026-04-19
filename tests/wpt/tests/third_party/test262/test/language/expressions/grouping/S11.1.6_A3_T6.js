// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "\"This\" operator only evaluates Expression"
es5id: 11.1.6_A3_T6
description: Applying grouping operator to delete operator
flags: [noStrict]
---*/

//CHECK#1
if (delete (x) !== true) {
  throw new Test262Error('#1: delete (x) === true');
}
