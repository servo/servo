// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White Space and Line Terminator between MemberExpression and Arguments
    are allowed
es5id: 11.2.3_A1
description: Checking by using eval
---*/

//CHECK#1
if (eval("Number\u0009()") !== 0) {
  throw new Test262Error('#1: Number\\u0009() === 0');
}

//CHECK#2
if (eval("Number\u000B()") !== 0) {
  throw new Test262Error('#2: Number\\u000B() === 0');  
}

//CHECK#3
if (eval("Number\u000C()") !== 0) {
  throw new Test262Error('#3: Number\\u000C() === 0');
}

//CHECK#4
if (eval("Number\u0020()") !== 0) {
  throw new Test262Error('#4: Number\\u0020 === 0');
}

//CHECK#5
if (eval("Number\u00A0()") !== 0) {
  throw new Test262Error('#5: Number\\u00A0() === 0');
}

//CHECK#6
if (eval("Number\u000A()") !== 0) {
  throw new Test262Error('#6: Number\\u000A() === 0');  
}

//CHECK#7
if (eval("Number\u000D()") !== 0) {
  throw new Test262Error('#7: Number\\u000D() === 0');
}

//CHECK#8
if (eval("Number\u2028()") !== 0) {
  throw new Test262Error('#8: Number\\u2028() === 0');
}

//CHECK#9
if (eval("Number\u2029()") !== 0) {
  throw new Test262Error('#9: Number\\u2029() === 0');
}

//CHECK#10
if (eval("Number\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u2029()") !== 0) {
  throw new Test262Error('#10: Number\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029() === 0');
}
