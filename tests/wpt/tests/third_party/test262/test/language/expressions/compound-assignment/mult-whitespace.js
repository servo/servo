// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White Space and Line Terminator between LeftHandSideExpression and "@="
    or between "@=" and AssignmentExpression are allowed
es5id: 11.13.2_A1_T1
esid: sec-assignment-operators
description: Checking by using eval, check operator is x *= y
---*/

var x;

x = -1;
assert.sameValue(x	*=	-1, 1, 'U+0009 (expression)');
assert.sameValue(x, 1, 'U+0009 (side effect)');

x = -1;
assert.sameValue(x*=-1, 1, 'U+000B (expression)');
assert.sameValue(x, 1, 'U+000B (side effect)');

x = -1;
assert.sameValue(x*=-1, 1, 'U+000C (expression)');
assert.sameValue(x, 1, 'U+000C (side effect)');

x = -1;
assert.sameValue(x *= -1, 1, 'U+0020 (expression)');
assert.sameValue(x, 1, 'U+0020 (side effect)');

x = -1;
assert.sameValue(x *= -1, 1, 'U+00A0 (expression)');
assert.sameValue(x, 1, 'U+00A0 (side effect)');

x = -1;
assert.sameValue(x
*=
-1, 1, 'U+000A (expression)');
assert.sameValue(x, 1, 'U+000A (side effect)');

x = -1;
assert.sameValue(x*=-1, 1, 'U+000D (expression)');
assert.sameValue(x, 1, 'U+000D (side effect)');

x = -1;
assert.sameValue(x *= -1, 1, 'U+2028 (expression)');
assert.sameValue(x, 1, 'U+2028 (side effect)');

x = -1;
assert.sameValue(x *= -1, 1, 'U+2029 (expression)');
assert.sameValue(x, 1, 'U+2029 (side effect)');

x = -1;
assert.sameValue(x	  
  *=	  
  -1, 1, 'U+0009U+000BU+000CU+0020U+00A0U+000AU+000DU+2028U+2029 (expression)');
assert.sameValue(x, 1, 'U+0009U+000BU+000CU+0020U+00A0U+000AU+000DU+2028U+2029 (side effect)');
