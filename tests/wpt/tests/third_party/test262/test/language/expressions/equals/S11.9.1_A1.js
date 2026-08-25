// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White Space and Line Terminator between EqualityExpression and "==" or
    between "==" and RelationalExpression are allowed
es5id: 11.9.1_A1
description: Checking by using eval
---*/

//CHECK#1
if (eval("true\u0009==\u00091") !== true) {
  throw new Test262Error('#1: (true\\u0009==\\u00091) === true');
}

//CHECK#2
if (eval("true\u000B==\u000B1") !== true) {
  throw new Test262Error('#2: (true\\u000B==\\u000B1) === true');  
}

//CHECK#3
if (eval("true\u000C==\u000C1") !== true) {
  throw new Test262Error('#3: (true\\u000C==\\u000C1) === true');
}

//CHECK#4
if (eval("true\u0020==\u00201") !== true) {
  throw new Test262Error('#4: (true\\u0020==\\u00201) === true');
}

//CHECK#5
if (eval("true\u00A0==\u00A01") !== true) {
  throw new Test262Error('#5: (true\\u00A0==\\u00A01) === true');
}

//CHECK#6
if (eval("true\u000A==\u000A1") !== true) {
  throw new Test262Error('#6: (true\\u000A==\\u000A1) === true');  
}

//CHECK#7
if (eval("true\u000D==\u000D1") !== true) {
  throw new Test262Error('#7: (true\\u000D==\\u000D1) === true');
}

//CHECK#8
if (eval("true\u2028==\u20281") !== true) {
  throw new Test262Error('#8: (true\\u2028==\\u20281) === true');
}

//CHECK#9
if (eval("true\u2029==\u20291") !== true) {
  throw new Test262Error('#9: (true\\u2029==\\u20291) === true');
}

//CHECK#10
if (eval("true\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u2029==\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u20291") !== true) {
  throw new Test262Error('#10: (true\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029==\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u20291) === true');
}
