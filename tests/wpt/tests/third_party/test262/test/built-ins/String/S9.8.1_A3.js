// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If m is less than zero, return the string concatenation of the
    string "-" and ToString(-m)
es5id: 9.8.1_A3
description: -1234567890 convert to String by explicit transformation
---*/

// CHECK#1
if (String(-1234567890) !== "-1234567890") {
  throw new Test262Error('#1: String(-1234567890) === "-1234567890". Actual: ' + (String(-1234567890)));
}

// CHECK#2
if ("-" + String(-(-1234567890)) !== "-1234567890") {
  throw new Test262Error('#2: "-"+String(-(-1234567890)) === "-1234567890". Actual: ' + ("-" + String(-(-1234567890))));
}
