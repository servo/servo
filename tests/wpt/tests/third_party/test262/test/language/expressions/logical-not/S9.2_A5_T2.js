// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of boolean conversion from nonempty string value (length is not
    zero) is true; from empty String (length is zero) is false
es5id: 9.2_A5_T2
description: "\"\" convert to Boolean by implicit transformation"
---*/

// CHECK#1
if (!("") !== true) {
  throw new Test262Error('#1: !("") === true. Actual: ' + (!("")));
}
