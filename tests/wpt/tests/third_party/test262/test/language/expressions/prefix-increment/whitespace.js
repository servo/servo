// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
    White Space and Line Terminator between "++" and UnaryExpression are
    allowed
es5id: 11.4.4_A1
esid: sec-prefix-increment-operator
---*/

var x = 0;

assert.sameValue(++	x, 1, 'U+0009 (expression)');
assert.sameValue(x, 1, 'U+0009 (side effect)');

assert.sameValue(++x, 2, 'U+000B (expression)');
assert.sameValue(x, 2, 'U+000B (side effect)');

assert.sameValue(++x, 3, 'U+000C (expression)');
assert.sameValue(x, 3, 'U+000C (side effect)');

assert.sameValue(++ x, 4, 'U+0020 (expression)');
assert.sameValue(x, 4, 'U+0020 (side effect)');

assert.sameValue(++ x, 5, 'U+00A0 (expression)');
assert.sameValue(x, 5, 'U+00A0 (side effect)');

assert.sameValue(++
x, 6, 'U+000A (expression)');
assert.sameValue(x, 6, 'U+000A (side effect)');

assert.sameValue(++x, 7, 'U+000D (expression)');
assert.sameValue(x, 7, 'U+000D (side effect)');

assert.sameValue(++ x, 8, 'U+2028 (expression)');
assert.sameValue(x, 8, 'U+2028 (side effect)');

assert.sameValue(++ x, 9, 'U+2029 (expression)');
assert.sameValue(x, 9, 'U+2029 (side effect)');

assert.sameValue(
  ++	  
  x,
  10,
  'U+0009U+000BU+000CU+0020U+00A0U+000AU+000DU+2028U+2029 (expression)'
);
assert.sameValue(
  x, 10, 'U+0009U+000BU+000CU+0020U+00A0U+000AU+000DU+2028U+2029 (side effect)'
);
