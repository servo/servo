// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Global object has properties such as built-in objects such as
    Math, String, Date, parseInt, etc
es5id: 10.2.3_A1.1_T2
description: Global execution context - Function Properties
---*/

//CHECK#4
if (eval === null) {
  throw new Test262Error("#4: eval === null");
}

//CHECK#5
if (parseInt === null) {
  throw new Test262Error("#5: parseInt === null");
}

//CHECK#6
if (parseFloat === null) {
  throw new Test262Error("#6: parseFloat === null");
}

//CHECK#7
if (isNaN === null) {
  throw new Test262Error("#7: isNaN === null");
}

//CHECK#8
if (isFinite === null) {
  throw new Test262Error("#8: isFinite === null");
}

//CHECK#9
if (decodeURI === null) {
  throw new Test262Error("#9: decodeURI === null");
}

//CHECK#10
if (decodeURIComponent === null) {
  throw new Test262Error("#10: decodeURIComponent === null");
}

//CHECK#11
if (encodeURI === null) {
  throw new Test262Error("#11: encodeURI === null");
}

//CHECK#12
if (encodeURIComponent === null) {
  throw new Test262Error("#12: encodeURIComponent === null");
}
