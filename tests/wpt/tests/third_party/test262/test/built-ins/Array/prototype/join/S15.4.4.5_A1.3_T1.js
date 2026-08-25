// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If array element is undefined or null, use the empty string
esid: sec-array.prototype.join
description: Checking this use new Array() and []
---*/

var x = [];
x[0] = undefined;
if (x.join() !== "") {
  throw new Test262Error('#1: x = []; x[0] = undefined; x.join() === "". Actual: ' + (x.join()));
}

x = [];
x[0] = null;
if (x.join() !== "") {
  throw new Test262Error('#2: x = []; x[0] = null; x.join() === "". Actual: ' + (x.join()));
}

x = Array(undefined, 1, null, 3);
if (x.join() !== ",1,,3") {
  throw new Test262Error('#3: x = Array(undefined,1,null,3); x.join() === ",1,,3". Actual: ' + (x.join()));
}
