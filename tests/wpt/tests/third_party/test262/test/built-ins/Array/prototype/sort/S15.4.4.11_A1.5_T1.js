// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If comparefn is undefined, use SortCompare operator
esid: sec-array.prototype.sort
description: Checking sort() and sort(undefined)
---*/

var x = new Array(1, 0);
x.sort();

if (x.length !== 2) {
  throw new Test262Error('#1: var x = new Array(1,0);  x.sort(); x.length === 2. Actual: ' + (x.length));
}

if (x[0] !== 0) {
  throw new Test262Error('#2: var x = new Array(1,0);  x.sort(); x[0] === 0. Actual: ' + (x[0]));
}

if (x[1] !== 1) {
  throw new Test262Error('#3: var x = new Array(1,0);  x.sort(); x[1] === 1. Actual: ' + (x[1]));
}

var x = new Array(1, 0);
x.sort(undefined);

if (x.length !== 2) {
  throw new Test262Error('#4: var x = new Array(1,0);  x.sort(undefined); x.length === 2. Actual: ' + (x.length));
}

if (x[0] !== 0) {
  throw new Test262Error('#5: var x = new Array(1,0);  x.sort(undefined); x[0] === 0. Actual: ' + (x[0]));
}

if (x[1] !== 1) {
  throw new Test262Error('#6: var x = new Array(1,0);  x.sort(undefined); x[1] === 1. Actual: ' + (x[1]));
}
