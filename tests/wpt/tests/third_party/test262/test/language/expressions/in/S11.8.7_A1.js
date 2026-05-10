// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White Space and Line Terminator between RelationalExpression and "in" and
    between "in" and ShiftExpression are allowed
es5id: 11.8.7_A1
description: Checking by using eval
---*/

//CHECK#1
if (eval("'MAX_VALUE'\u0009in\u0009Number") !== true) {
  throw new Test262Error('#1: "MAX_VALUE"\\u0009in\\u0009Number === true');
}

//CHECK#2
if (eval("'MAX_VALUE'\u000Bin\u000BNumber") !== true) {
  throw new Test262Error('#2: "MAX_VALUE"\\u000Bin\\u000BNumber === true');  
}

//CHECK#3
if (eval("'MAX_VALUE'\u000Cin\u000CNumber") !== true) {
  throw new Test262Error('#3: "MAX_VALUE"\\u000Cin\\u000CNumber === true');
}

//CHECK#4
if (eval("'MAX_VALUE'\u0020in\u0020Number") !== true) {
  throw new Test262Error('#4: "MAX_VALUE"\\u0020in\\u0020Number === true');
}

//CHECK#5
if (eval("'MAX_VALUE'\u00A0in\u00A0Number") !== true) {
  throw new Test262Error('#5: "MAX_VALUE"\\u00A0in\\u00A0Number === true');
}

//CHECK#6
if (eval("'MAX_VALUE'\u000Ain\u000ANumber") !== true) {
  throw new Test262Error('#6: "MAX_VALUE"\\u000Ain\\u000ANumber === true');  
}

//CHECK#7
if (eval("'MAX_VALUE'\u000Din\u000DNumber") !== true) {
  throw new Test262Error('#7: "MAX_VALUE"\\u000Din\\u000DNumber === true');
}

//CHECK#8
if (eval("'MAX_VALUE'\u2028in\u2028Number") !== true) {
  throw new Test262Error('#8: "MAX_VALUE"\\u2028in\\u2028Number === true');
}

//CHECK#9
if (eval("'MAX_VALUE'\u2029in\u2029Number") !== true) {
  throw new Test262Error('#9: "MAX_VALUE"\\u2029in\\u2029Number === true');
}

//CHECK#10
if (eval("'MAX_VALUE'\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u2029in\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u2029Number") !== true) {
  throw new Test262Error('#10: "MAX_VALUE"\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029in\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029Number === true');
}
