// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If value is NaN, +0, -0, +Infinity, or -Infinity, return +0
es5id: 9.7_A1
description: >
    For testing use String.fromCharCode(Number).charCodeAt(0)
    construction
---*/

// CHECK#1
if (String.fromCharCode(Number.NaN).charCodeAt(0) !== +0) {
  throw new Test262Error('#1.1: String.fromCharCode(Number.NaN).charCodeAt(0) === 0. Actual: ' + (String.fromCharCode(Number.NaN).charCodeAt(0)));
} else if (1 / String.fromCharCode(Number.NaN).charCodeAt(0) !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#1.2: String.fromCharCode(Number.NaN).charCodeAt(0) === +0. Actual: -0');
}

// CHECK#2
if (String.fromCharCode(Number("abc")).charCodeAt(0) !== +0) {
  throw new Test262Error('#2.1: String.fromCharCode(Number("abc")).charCodeAt(0) === 0. Actual: ' + (String.fromCharCode(Number("abc")).charCodeAt(0)));
} else if (1 / String.fromCharCode(0).charCodeAt(0) !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#2.2: String.fromCharCode(0).charCodeAt(0) === +0. Actual: -0');
}

// CHECK#3
if (String.fromCharCode(0).charCodeAt(0) !== +0) {
  throw new Test262Error('#3.1: String.fromCharCode(0).charCodeAt(0) === 0. Actual: ' + (String.fromCharCode(0).charCodeAt(0)));
} else if (1 / String.fromCharCode(0).charCodeAt(0) !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#3.2: String.fromCharCode(0).charCodeAt(0) === +0. Actual: -0');
}

// CHECK#4
if (String.fromCharCode(-0).charCodeAt(0) !== +0) {
  throw new Test262Error("#4.1: String.fromCharCode(-0).charCodeAt(0) === +0");
} else if (1 / String.fromCharCode(-0).charCodeAt(0) !== Number.POSITIVE_INFINITY) {
  throw new Test262Error("#4.2: String.fromCharCode(-0).charCodeAt(0) === +0. Actual: -0");
}

// CHECK#5
if (String.fromCharCode(Number.POSITIVE_INFINITY).charCodeAt(0) !== +0) {
  throw new Test262Error('#5.1: String.fromCharCode(Number.POSITIVE_INFINITY).charCodeAt(0) === 0. Actual: ' + (String.fromCharCode(Number.POSITIVE_INFINITY).charCodeAt(0)));
} else if (1 / String.fromCharCode(Number.POSITIVE_INFINITY).charCodeAt(0) !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#5.2: String.fromCharCode(Number.POSITIVE_INFINITY).charCodeAt(0) === +0. Actual: -0');
}

// CHECK#6
if (String.fromCharCode(Number.NEGATIVE_INFINITY).charCodeAt(0) !== +0) {
  throw new Test262Error("#6.1: String.fromCharCode(Number.NEGATIVE_INFINITY).charCodeAt(0) === +0");
} else if (1 / String.fromCharCode(Number.NEGATIVE_INFINITY).charCodeAt(0) !== Number.POSITIVE_INFINITY) {
  throw new Test262Error("#6.2: String.fromCharCode(Number.NEGATIVE_INFINITY).charCodeAt(0) === +0. Actual: -0");
}
