// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tostring
info: |
    The result of calling this function is the same as if
    the built-in join method were invoked for this object with no argument
es5id: 15.4.4.2_A1_T1
description: If Result(2) is zero, return the empty string
---*/

var x = new Array();
if (x.toString() !== x.join()) {
  throw new Test262Error('#1.1: x = new Array(); x.toString() === x.join(). Actual: ' + (x.toString()));
} else {
  if (x.toString() !== "") {
    throw new Test262Error('#1.2: x = new Array(); x.toString() === "". Actual: ' + (x.toString()));
  }
}

x = [];
x[0] = 1;
x.length = 0;
if (x.toString() !== x.join()) {
  throw new Test262Error('#2.1: x = []; x[0] = 1; x.length = 0; x.toString() === x.join(). Actual: ' + (x.toString()));
} else {
  if (x.toString() !== "") {
    throw new Test262Error('#2.2: x = []; x[0] = 1; x.length = 0; x.toString() === "". Actual: ' + (x.toString()));
  }
}
