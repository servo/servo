// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: ToUint16 returns values between 0 and 2^16-1
es5id: 9.7_A2.1
description: >
    Converting numbers, which are in\outside of Uint16 scopes, with
    String.fromCharCode(Number).charCodeAt(0) construction
---*/

// CHECK#1
if (String.fromCharCode(0).charCodeAt(0) !== 0) {
  throw new Test262Error('#1: String.fromCharCode(0).charCodeAt(0) === 0. Actual: ' + (String.fromCharCode(0).charCodeAt(0)));
}

// CHECK#2
if (String.fromCharCode(1).charCodeAt(0) !== 1) {
  throw new Test262Error('#2: String.fromCharCode(1).charCodeAt(0) === 1. Actual: ' + (String.fromCharCode(1).charCodeAt(0)));
}

// CHECK#3
if (String.fromCharCode(-1).charCodeAt(0) !== 65535) {
  throw new Test262Error('#3: String.fromCharCode(-1).charCodeAt(0) === 65535. Actual: ' + (String.fromCharCode(-1).charCodeAt(0)));
}

// CHECK#4
if (String.fromCharCode(65535).charCodeAt(0) !== 65535) {
  throw new Test262Error('#4: String.fromCharCode(65535).charCodeAt(0) === 65535. Actual: ' + (String.fromCharCode(65535).charCodeAt(0)));
}

// CHECK#5
if (String.fromCharCode(65534).charCodeAt(0) !== 65534) {
  throw new Test262Error('#5: String.fromCharCode(65534).charCodeAt(0) === 65534. Actual: ' + (String.fromCharCode(65534).charCodeAt(0)));
}

// CHECK#6
if (String.fromCharCode(65536).charCodeAt(0) !== 0) {
  throw new Test262Error('#6: String.fromCharCode(65536).charCodeAt(0) === 0. Actual: ' + (String.fromCharCode(65536).charCodeAt(0)));
}

// CHECK#7
if (String.fromCharCode(4294967295).charCodeAt(0) !== 65535) {
  throw new Test262Error('#7: String.fromCharCode(4294967295).charCodeAt(0) === 65535. Actual: ' + (String.fromCharCode(4294967295).charCodeAt(0)));
}

// CHECK#8
if (String.fromCharCode(4294967294).charCodeAt(0) !== 65534) {
  throw new Test262Error('#8: String.fromCharCode(4294967294).charCodeAt(0) === 65534. Actual: ' + (String.fromCharCode(4294967294).charCodeAt(0)));
}

// CHECK#9
if (String.fromCharCode(4294967296).charCodeAt(0) !== 0) {
  throw new Test262Error('#9: String.fromCharCode(4294967296).charCodeAt(0) === 0. Actual: ' + (String.fromCharCode(4294967296).charCodeAt(0)));
}
