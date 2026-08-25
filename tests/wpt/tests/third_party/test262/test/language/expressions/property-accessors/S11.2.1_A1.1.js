// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White Space and Line Terminator between MemberExpression or
    CallExpression and "." and between "." and Identifier are allowed
es5id: 11.2.1_A1.1
description: Checking by using eval
---*/

//CHECK#1
if (eval("Number\u0009.\u0009POSITIVE_INFINITY") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#1: Number\\u0009.\\u0009POSITIVE_INFINITY === Number.POSITIVE_INFINITY');
}

//CHECK#2
if (eval("Number\u000B.\u000BPOSITIVE_INFINITY") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#2: Number\\u000B.\\u000BPOSITIVE_INFINITY === Number.POSITIVE_INFINITY');  
}

//CHECK#3
if (eval("Number\u000C.\u000CPOSITIVE_INFINITY") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#3: Number\\u000C.\\u000CPOSITIVE_INFINITY === Number.POSITIVE_INFINITY');
}

//CHECK#4
if (eval("Number\u0020.\u0020POSITIVE_INFINITY") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#4: Number\\u0020.\\u0020POSITIVE_INFINITY === Number.POSITIVE_INFINITY');
}

//CHECK#5
if (eval("Number\u00A0.\u00A0POSITIVE_INFINITY") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#5: Number\\u00A0.\\u00A0POSITIVE_INFINITY === Number.POSITIVE_INFINITY');
}

//CHECK#6
if (eval("Number\u000A.\u000APOSITIVE_INFINITY") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#6: Number\\u000A.\\u000APOSITIVE_INFINITY === Number.POSITIVE_INFINITY');  
}

//CHECK#7
if (eval("Number\u000D.\u000DPOSITIVE_INFINITY") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#7: Number\\u000D.\\u000DPOSITIVE_INFINITY === Number.POSITIVE_INFINITY');
}

//CHECK#8
if (eval("Number\u2028.\u2028POSITIVE_INFINITY") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#8: Number\\u2028.\\u2028POSITIVE_INFINITY === Number.POSITIVE_INFINITY');
}

//CHECK#9
if (eval("Number\u2029.\u2029POSITIVE_INFINITY") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#9: Number\\u2029.\\u2029POSITIVE_INFINITY === Number.POSITIVE_INFINITY');
}

//CHECK#10
if (eval("Number\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u2029.\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u2029POSITIVE_INFINITY") !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#10: Number\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029.\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029POSITIVE_INFINITY === Number.POSITIVE_INFINITY');
}
