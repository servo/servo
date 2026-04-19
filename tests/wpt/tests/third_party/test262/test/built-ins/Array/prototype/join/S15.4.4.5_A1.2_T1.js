// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If separator is undefined, a single comma is used as the separator
esid: sec-array.prototype.join
description: Checking this use new Array() and []
---*/

var x = new Array(0, 1, 2, 3);
if (x.join() !== "0,1,2,3") {
  throw new Test262Error('#1: x = new Array(0,1,2,3); x.join() === "0,1,2,3". Actual: ' + (x.join()));
}

x = [];
x[0] = 0;
x[3] = 3;
if (x.join() !== "0,,,3") {
  throw new Test262Error('#2: x = []; x[0] = 0; x[3] = 3; x.join() === "0,,,3". Actual: ' + (x.join()));
}

x = [];
x[0] = 0;
if (x.join() !== "0") {
  throw new Test262Error('#3: x = []; x[0] = 0; x.join() === "0". Actual: ' + (x.join()));
}
