// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The sort function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.sort
description: If comparefn is not undefined
---*/

var obj = {
  valueOf: function() {
    return 1
  },
  toString: function() {
    return -2
  }
};
var alphabetR = {
  0: undefined,
  1: 2,
  2: 1,
  3: "X",
  4: -1,
  5: "a",
  6: true,
  7: obj,
  8: NaN,
  9: Infinity
};
alphabetR.sort = Array.prototype.sort;
alphabetR.length = 10;
var alphabet = [true, "a", "X", NaN, Infinity, 2, 1, obj, -1, undefined];

var myComparefn = function(x, y) {
  var xS = String(x);
  var yS = String(y);
  if (xS < yS) return 1
  if (xS > yS) return -1;
  return 0;
}

alphabetR.sort(myComparefn);

alphabetR.getClass = Object.prototype.toString;
if (alphabetR.getClass() !== "[object " + "Object" + "]") {
  throw new Test262Error('#0: alphabetR.sort() is Object object, not Array object');
}

var result = true;
for (var i = 0; i < 10; i++) {
  if (!(isNaN(alphabetR[i]) && isNaN(alphabet[i]))) {
    if (alphabetR[i] !== alphabet[i]) result = false;
  }
}

if (result !== true) {
  throw new Test262Error('#1: Check ToString operator');
}
