// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White Space and Line Terminator between LogicalANDExpression and "&&" or
    between "&&" and BitwiseORExpression are allowed
es5id: 11.11.1_A1
description: Checking by using eval
---*/

//CHECK#1
if ((eval("true\u0009&&\u0009true")) !== true) {
  throw new Test262Error('#1: (true\\u0009&&\\u0009true) === true');
}

//CHECK#2
if ((eval("true\u000B&&\u000Btrue")) !== true) {
  throw new Test262Error('#2: (true\\u000B&&\\u000Btrue) === true');  
}

//CHECK#3
if ((eval("true\u000C&&\u000Ctrue")) !== true) {
  throw new Test262Error('#3: (true\\u000C&&\\u000Ctrue) === true');
}

//CHECK#4
if ((eval("true\u0020&&\u0020true")) !== true) {
  throw new Test262Error('#4: (true\\u0020&&\\u0020true) === true');
}

//CHECK#5
if ((eval("true\u00A0&&\u00A0true")) !== true) {
  throw new Test262Error('#5: (true\\u00A0&&\\u00A0true) === true');
}

//CHECK#6
if ((eval("true\u000A&&\u000Atrue")) !== true) {
  throw new Test262Error('#6: (true\\u000A&&\\u000Atrue) === true');  
}

//CHECK#7
if ((eval("true\u000D&&\u000Dtrue")) !== true) {
  throw new Test262Error('#7: (true\\u000D&&\\u000Dtrue) === true');
}

//CHECK#8
if ((eval("true\u2028&&\u2028true")) !== true) {
  throw new Test262Error('#8: (true\\u2028&&\\u2028true) === true');
}

//CHECK#9
if ((eval("true\u2029&&\u2029true")) !== true) {
  throw new Test262Error('#9: (true\\u2029&&\\u2029true) === true');
}


//CHECK#10
if ((eval("true\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u2029&&\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u2029true")) !== true) {
  throw new Test262Error('#10: (true\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029&&\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029true) === true');
}
