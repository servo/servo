// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-delete-operator
description: >
  White Space and Line Terminator between "delete" and UnaryExpression are allowed
info: |
  UnaryExpression :
      delete UnaryExpression

---*/

var result;

result = delete	0;
assert.sameValue(result, true, '\\u0009');

result = delete0;
assert.sameValue(result, true, '\\u000B');

result = delete0;
assert.sameValue(result, true, '\\u000C');

result = delete 0;
assert.sameValue(result, true, '\\u0020');

result = delete 0;
assert.sameValue(result, true, '\\u00A0');

// Line Break is intentional
result = delete
0;
assert.sameValue(result, true, '\\u000A');

// Line Break is intentional
result = delete
0;
assert.sameValue(result, true, '\\u000D');

result = delete 0;
assert.sameValue(result, true, '\\u2028');

result = delete 0;
assert.sameValue(result, true, '\\u2029');

// Line Break is intentional
result = delete	  
  0;
assert.sameValue(result, true, '\\u0009\\u000B\\u000C\\u0020\\u00A0\\u000A\\u000D\\u2028\\u2029');
