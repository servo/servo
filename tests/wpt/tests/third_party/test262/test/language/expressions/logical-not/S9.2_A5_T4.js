// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of boolean conversion from nonempty string value (length is not
    zero) is true; from empty String (length is zero) is false
es5id: 9.2_A5_T4
description: Any nonempty string convert to Boolean by implicit transformation
---*/

// CHECK#1
if (!(" ") !== false) {
  throw new Test262Error('#1: !(" ") === false. Actual: ' + (!(" ")));	
}

// CHECK#2
if (!("Nonempty String") !== false) {
  throw new Test262Error('#2: !("Nonempty String") === false. Actual: ' + (!("Nonempty String")));	
}
