// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "\"This\" operator only evaluates Expression"
es5id: 11.1.6_A3_T1
description: Applying grouping operator to Boolean
---*/

// Check for Boolean

//CHECK#1
if ((true) !== true) {
  throw new Test262Error('#1: (true) === true');
}

//CHECK#2
var x = new Boolean(true);
if ((x) !== x) {
  throw new Test262Error('#2: var x = new Boolean(true); (x) === x. Actual: ' + ((x)));
}
