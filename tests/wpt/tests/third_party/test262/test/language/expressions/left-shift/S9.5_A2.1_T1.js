// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: ToInt32 returns values between -2^31 and 2^31-1
es5id: 9.5_A2.1_T1
description: >
    Converting some numbers, which are in\outside of Int32 scopes,
    with <<0 operator
---*/

// CHECK#1
if ((-2147483647 << 0) !== -2147483647) {
  throw new Test262Error('#1: (-2147483647 << 0) === -2147483647. Actual: ' + ((-2147483647 << 0)));
}

// CHECK#2
if ((-2147483648 << 0) !== -2147483648) {
  throw new Test262Error('#2: (-2147483648 << 0) === -2147483648. Actual: ' + ((-2147483648 << 0)));
}

// CHECK#3
if ((-2147483649 << 0) !== 2147483647) {
  throw new Test262Error('#3: (-2147483649 << 0) === 2147483647. Actual: ' + ((-2147483649 << 0)));
}

// CHECK#4
if ((-4294967296 << 0) !== 0) {
  throw new Test262Error('#4: (-4294967296 << 0) === 0. Actual: ' + ((-4294967296 << 0)));
}

// CHECK#5
if ((2147483646 << 0) !== 2147483646) {
  throw new Test262Error('#5: (2147483646 << 0) === 2147483646. Actual: ' + ((2147483646 << 0)));
}

// CHECK#6
if ((2147483647 << 0) !== 2147483647) {
  throw new Test262Error('#6: (2147483647 << 0) === 2147483647. Actual: ' + ((2147483647 << 0)));
}

// CHECK#7
if ((2147483648 << 0) !== -2147483648) {
  throw new Test262Error('#7: (2147483648 << 0) === -2147483648. Actual: ' + ((2147483648 << 0)));
}

// CHECK#8
if ((4294967296 << 0) !== 0) {
  throw new Test262Error('#8: (4294967296 << 0) === 0. Actual: ' + ((4294967296 << 0)));
}
