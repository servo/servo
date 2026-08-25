// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Compute result modulo 2^16
es5id: 9.7_A2.2
description: >
    For testing use String.fromCharCode(Number).charCodeAt(0)
    construction
---*/

// CHECK#1
if (String.fromCharCode(-32767).charCodeAt(0) !== 32769) {
  throw new Test262Error('#1: String.fromCharCode(-32767).charCodeAt(0) === 32769. Actual: ' + (String.fromCharCode(-32767).charCodeAt(0)));
}

// CHECK#2
if (String.fromCharCode(-32768).charCodeAt(0) !== 32768) {
  throw new Test262Error('#2: String.fromCharCode(-32768).charCodeAt(0) === 32768. Actual: ' + (String.fromCharCode(-32768).charCodeAt(0)));
}

// CHECK#3
if (String.fromCharCode(-32769).charCodeAt(0) !== 32767) {
  throw new Test262Error('#3: String.fromCharCode(-32769).charCodeAt(0) === 32767. Actual: ' + (String.fromCharCode(-32769).charCodeAt(0)));
}

// CHECK#4
if (String.fromCharCode(-65535).charCodeAt(0) !== 1) {
  throw new Test262Error('#4: String.fromCharCode(-65535).charCodeAt(0) === 1. Actual: ' + (String.fromCharCode(-65535).charCodeAt(0)));
}

// CHECK#5
if (String.fromCharCode(-65536).charCodeAt(0) !== 0) {
  throw new Test262Error('#5: String.fromCharCode(-65536).charCodeAt(0) === 0. Actual: ' + (String.fromCharCode(-65536).charCodeAt(0)));
}

// CHECK#6
if (String.fromCharCode(-65537).charCodeAt(0) !== 65535) {
  throw new Test262Error('#6: String.fromCharCode(-65537).charCodeAt(0) === 65535. Actual: ' + (String.fromCharCode(-65537).charCodeAt(0)));
}

// CHECK#7
if (String.fromCharCode(65535).charCodeAt(0) !== 65535) {
  throw new Test262Error('#7: String.fromCharCode(65535).charCodeAt(0) === 65535. Actual: ' + (String.fromCharCode(65535).charCodeAt(0)));
}

// CHECK#8
if (String.fromCharCode(65536).charCodeAt(0) !== 0) {
  throw new Test262Error('#8: String.fromCharCode(65536).charCodeAt(0) === 0. Actual: ' + (String.fromCharCode(65536).charCodeAt(0)));
}

// CHECK#9
if (String.fromCharCode(65537).charCodeAt(0) !== 1) {
  throw new Test262Error('#9: String.fromCharCode(65537).charCodeAt(0) === 1. Actual: ' + (String.fromCharCode(65537).charCodeAt(0)));
}

// CHECK#10
if (String.fromCharCode(131071).charCodeAt(0) !== 65535) {
  throw new Test262Error('#10: String.fromCharCode(131071).charCodeAt(0) === 65535. Actual: ' + (String.fromCharCode(131071).charCodeAt(0)));
}

// CHECK#11
if (String.fromCharCode(131072).charCodeAt(0) !== 0) {
  throw new Test262Error('#11: String.fromCharCode(131072).charCodeAt(0) === 0. Actual: ' + (String.fromCharCode(131072).charCodeAt(0)));
}

// CHECK#12
if (String.fromCharCode(131073).charCodeAt(0) !== 1) {
  throw new Test262Error('#12: String.fromCharCode(131073).charCodeAt(0) === 1. Actual: ' + (String.fromCharCode(131073).charCodeAt(0)));
}
