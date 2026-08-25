// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White Space and Line Terminator between "void" and UnaryExpression are
    allowed
es5id: 11.4.2_A1
description: Checking by using eval
---*/

//CHECK#1
if (eval("void\u00090") !== undefined) {
  throw new Test262Error('#1: void\\u00090 === undefined');
}

//CHECK#2
if (eval("void\u000B0") !== undefined) {
  throw new Test262Error('#2: void\\u000B0 === undefined');  
}

//CHECK#3
if (eval("void\u000C0") !== undefined) {
  throw new Test262Error('#3: void\\u000C0 === undefined');
}

//CHECK#4
if (eval("void\u00200") !== undefined) {
  throw new Test262Error('#4: void\\u00200 === undefined');
}

//CHECK#5
if (eval("void\u00A00") !== undefined) {
  throw new Test262Error('#5: void\\u00A00 === undefined');
}

//CHECK#6
if (eval("void\u000A0") !== undefined) {
  throw new Test262Error('#6: void\\u000A0 === undefined');  
}

//CHECK#7
if (eval("void\u000D0") !== undefined) {
  throw new Test262Error('#7: void\\u000D0 === undefined');
}

//CHECK#8
if (eval("void\u20280") !== undefined) {
  throw new Test262Error('#8: void\\u20280 === undefined');
}

//CHECK#9
if (eval("void\u20290") !== undefined) {
  throw new Test262Error('#9: void\\u20290 === undefined');
}

//CHECK#10
if (eval("void\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u20290") !== undefined) {
  throw new Test262Error('#10: void\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u20290 === undefined');
}
