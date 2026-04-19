// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    White Space between LeftHandSideExpression and "=" or between "=" and
    AssignmentExpression is allowed
es5id: 11.13.1_A1
---*/

var x;

x	=	'U+0009';
if (x !== 'U+0009') {
  throw new Test262Error('#1: (x\\u0009=\\u0009true) === true');
}

x='U+000B';
if (x !== 'U+000B') {
  throw new Test262Error('#2: (x\\u000B=\\u000Btrue) === true');
}

x='U+000C';
if (x !== 'U+000C') {
  throw new Test262Error('#3: (x\\u000C=\\u000Ctrue) === true');
}

x = 'U+0020';
if (x !== 'U+0020') {
  throw new Test262Error('#4: (x\\u0020=\\u0020true) === true');
}

x = 'U+00A0';
if (x !== 'U+00A0') {
  throw new Test262Error('#5: (x\\u00A0=\\u00A0true) === true');
}

x
=
'U+000D';
if (x !== 'U+000D') {
  throw new Test262Error('#7: (x\\u000D=\\u000Dtrue) === true');
}

x = 'U+2028';
if (x !== 'U+2028') {
  throw new Test262Error('#8: (x\\u2028=\\u2028true) === true');
}

x = 'U+2029';
if (x !== 'U+2029') {
  throw new Test262Error('#9: (x\\u2029=\\u2029true) === true');
}

x	  
  =	  
  'U+0009U+000BU+000CU+0020U+00A0U+000DU+2028U+2029';
if (x !== 'U+0009U+000BU+000CU+0020U+00A0U+000DU+2028U+2029') {
  throw new Test262Error('#10: (x\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000D\\u2028\\u2029=\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000D\\u2028\\u2029true) === true');
}
