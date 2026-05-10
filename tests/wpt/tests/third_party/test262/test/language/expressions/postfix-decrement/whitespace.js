// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: White Space between LeftHandSideExpression and "--" are allowed
es5id: 11.3.2_A1.2_T1
esid: sec-postfix-decrement-operator
---*/

var x = 0;

assert.sameValue(x	--, 0, 'U+0009 (expression)');
assert.sameValue(x, -1, 'U+0009 (side effect)');

assert.sameValue(x--, -1, 'U+000B (expression)');
assert.sameValue(x, -2, 'U+000B (side effect)');

assert.sameValue(x--, -2, 'U+000C (expression)');
assert.sameValue(x, -3, 'U+000C (side effect)');

assert.sameValue(x --, -3, 'U+0020 (expression)');
assert.sameValue(x, -4, 'U+0020 (side effect)');

assert.sameValue(x --, -4, 'U+00A0 (expression)');
assert.sameValue(x, -5, 'U+00A0 (side effect)');

assert.sameValue(x	  --, -5, 'U+0009U+000BU+000CU+0020U+00A0 (expression)');
assert.sameValue(x, -6, 'U+0009U+000BU+000CU+0020U+00A0 (side effect)');
