// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If this object does not have a property named by ToString(j),
    and this object does not have a property named by ToString(k), return +0
esid: sec-array.prototype.sort
description: If comparefn is undefined, use SortCompare operator
---*/

var x = new Array(2);
x.sort();

if (x.length !== 2) {
  throw new Test262Error('#1: var x = new Array(2); x.sort(); x.length === 2. Actual: ' + (x.length));
}

if (x[0] !== undefined) {
  throw new Test262Error('#2: var x = new Array(2); x.sort(); x[0] === undefined. Actual: ' + (x[0]));
}

if (x[1] !== undefined) {
  throw new Test262Error('#3: var x = new Array(2); x.sort(); x[1] === undefined. Actual: ' + (x[1]));
}
