// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White Space and Line Terminator between "!" and UnaryExpression are
    allowed
es5id: 11.4.9_A1
description: Checking by using eval
---*/

//CHECK#1
if (eval("!\u0009true") !== false) {
  throw new Test262Error('#true: !\\u0009true === false');
}

//CHECK#2
if (eval("!\u000Btrue") !== false) {
  throw new Test262Error('#2: !\\u000Btrue === false');  
}

//CHECK#3
if (eval("!\u000Ctrue") !== false) {
  throw new Test262Error('#3: !\\u000Ctrue === false');
}

//CHECK#4
if (eval("!\u0020true") !== false) {
  throw new Test262Error('#4: !\\u0020 === false');
}

//CHECK#5
if (eval("!\u00A0true") !== false) {
  throw new Test262Error('#5: !\\u00A0true === false');
}

//CHECK#6
if (eval("!\u000Atrue") !== false) {
  throw new Test262Error('#6: !\\u000Atrue === false');  
}

//CHECK#7
if (eval("!\u000Dtrue") !== false) {
  throw new Test262Error('#7: !\\u000Dtrue === false');
}

//CHECK#8
if (eval("!\u2028true") !== false) {
  throw new Test262Error('#8: !\\u2028true === false');
}

//CHECK#9
if (eval("!\u2029true") !== false) {
  throw new Test262Error('#9: !\\u2029true === false');
}

//CHECK#10
if (eval("!\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u2029true") !== false) {
  throw new Test262Error('#10: !\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029true === false');
}
