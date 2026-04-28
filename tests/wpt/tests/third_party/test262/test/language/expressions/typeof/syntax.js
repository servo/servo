// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White Space and Line Terminator between "typeof" and UnaryExpression are
    allowed
es5id: 11.4.3_A1
es6id: 12.5.6.1
description: Checking by using eval
---*/

assert.sameValue(
  eval("var x = 0; typeof\u0009x"),
  "number",
  '#1: var x = 0; typeof\\u0009x; x === "number".'
);

assert.sameValue(
  eval("var x = 0; typeof\u000Bx"),
  "number",
  '#2: var x = 0; typeof\\u000Bx; x === "number".'
);

assert.sameValue(
  eval("var x = 0; typeof\u000Cx"),
  "number",
  '#3: var x = 0; typeof\\u000Cx; x === "number".'
);

assert.sameValue(
  eval("var x = 0; typeof\u0020x"),
  "number",
  '#4: var x = 0; typeof\\u0020x; x === "number".'
);

assert.sameValue(
  eval("var x = 0; typeof\u00A0x"),
  "number",
  '#5: var x = 0; typeof\\u00A0x; x === "number".'
);

assert.sameValue(
  eval("var x = 0; typeof\u000Ax"),
  "number",
  '#6: var x = 0; typeof\\u000Ax; x === "number".'
);

assert.sameValue(
  eval("var x = 0; typeof\u000Dx"),
  "number",
  '#7: var x = 0; typeof\\u000Dx; x === "number".'
);

assert.sameValue(
  eval("var x = 0; typeof\u2028x"),
  "number",
  '#8: var x = 0; typeof\\u2028x; x === "number".'
);

assert.sameValue(
  eval("var x = 0; typeof\u2029x"),
  "number",
  '#9: var x = 0; typeof\\u2029x; x === "number".'
);

assert.sameValue(
  eval("var x = 0; typeof\u0009\u000B\u000C\u0020\u00A0\u000A\u000D\u2028\u2029x"),
  "number",
  '#10: var x = 0; typeof\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029x; x === "number".'
);

assert.sameValue(
  eval("typeof(0)"),
  "number",
  'applied with grouping operator enclosing operand'
);
