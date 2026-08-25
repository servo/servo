// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If length is zero, return the empty string
esid: sec-array.prototype.join
description: Checking this use new Array() and []
---*/

var x = new Array();
if (x.join() !== "") {
  throw new Test262Error('#1: x = new Array(); x.join() === "". Actual: ' + (x.join()));
}

x = [];
x[0] = 1;
x.length = 0;
if (x.join() !== "") {
  throw new Test262Error('#2: x = []; x[0] = 1; x.length = 0; x.join() === "". Actual: ' + (x.join()));
}
