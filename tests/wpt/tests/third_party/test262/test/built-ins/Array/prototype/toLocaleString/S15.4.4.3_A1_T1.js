// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tolocalestring
info: |
    The elements of the array are converted to strings using their
    toLocaleString methods, and these strings are then concatenated, separated
    by occurrences of a separator string that has been derived in an
    implementation-defined locale-specific way
es5id: 15.4.4.3_A1_T1
description: it is the function that should be invoked
---*/

var n = 0;
var obj = {
  toLocaleString: function() {
    n++
  }
};
var arr = [undefined, obj, null, obj, obj];
arr.toLocaleString();

if (n !== 3) {
  throw new Test262Error('#1: var n = 0; var obj = {toLocaleString: function() {n++}}; var arr = [undefined, obj, null, obj, obj]; arr.toLocaleString(); n === 3. Actual: ' + (n));
}
