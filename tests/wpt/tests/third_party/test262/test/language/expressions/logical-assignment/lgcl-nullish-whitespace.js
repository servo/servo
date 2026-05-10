// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    White Space and Line Terminator between LeftHandSideExpression and "@="
    or between "@=" and AssignmentExpression are allowed
esid: sec-assignment-operators
description: Checking by evaluating expression "x[...]??=[...]y"
features: [logical-assignment-operators]
---*/
var x;

x = null;
assert.sameValue(x	??=	1, 1, 'U+0009 (expression)');
assert.sameValue(x, 1, 'U+0009 (side effect)');

x = null;
assert.sameValue(x??=1, 1, 'U+000B (expression)');
assert.sameValue(x, 1, 'U+000B (side effect)');

x = null;
assert.sameValue(x??=1, 1, 'U+000C (expression)');
assert.sameValue(x, 1, 'U+000C (side effect)');

x = null;
assert.sameValue(x ??= 1, 1, 'U+0020 (expression)');
assert.sameValue(x, 1, 'U+0020 (side effect)');

x = null;
assert.sameValue(x ??= 1, 1, 'U+00A0 (expression)');
assert.sameValue(x, 1, 'U+00A0 (side effect)');

x = null;
assert.sameValue(x
??=
1, 1, 'U+000A (expression)');
assert.sameValue(x, 1, 'U+000A (side effect)');

x = null;
assert.sameValue(x
??=
1, 1, 'U+000D (expression)');
assert.sameValue(x, 1, 'U+000D (side effect)');

x = null;
assert.sameValue(x ??= 1, 1, 'U+2028 (expression)');
assert.sameValue(x, 1, 'U+2028 (side effect)');

x = null;
assert.sameValue(x ??= 1, 1, 'U+2029 (expression)');
assert.sameValue(x, 1, 'U+2029 (side effect)');

x = null;
assert.sameValue(x	  
  ??=	  
  1, 1, 'U+0009U+000BU+000CU+0020U+00A0U+000AU+000DU+2028U+2029 (expression)');
assert.sameValue(x, 1, 'U+0009U+000BU+000CU+0020U+00A0U+000AU+000DU+2028U+2029 (side effect)');
