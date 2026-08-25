// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If [[Get]] ToString(j) is undefined, return 1.
    If [[]Get] ToString(k) is undefined, return -1
esid: sec-array.prototype.sort
description: If comparefn is undefined, use SortCompare operator
---*/

var x = new Array(undefined, 1);
x.sort();

if (x.length !== 2) {
  throw new Test262Error('#1: var x = new Array(undefined, 1); x.sort(); x.length === 2. Actual: ' + (x.length));
}

if (x[0] !== 1) {
  throw new Test262Error('#2: var x = new Array(undefined, 1); x.sort(); x[0] === 1. Actual: ' + (x[0]));
}

if (x[1] !== undefined) {
  throw new Test262Error('#3: var x = new Array(undefined, 1); x.sort(); x[1] === undefined. Actual: ' + (x[1]));
}

var x = new Array(1, undefined);
x.sort();

if (x.length !== 2) {
  throw new Test262Error('#4: var x = new Array(1, undefined); x.sort(); x.length === 2. Actual: ' + (x.length));
}

if (x[0] !== 1) {
  throw new Test262Error('#5: var x = new Array(1, undefined); x.sort(); x[0] === 1. Actual: ' + (x[0]));
}

if (x[1] !== undefined) {
  throw new Test262Error('#6: var x = new Array(1, undefined); x.sort(); x[1] === undefined. Actual: ' + (x[1]));
}
