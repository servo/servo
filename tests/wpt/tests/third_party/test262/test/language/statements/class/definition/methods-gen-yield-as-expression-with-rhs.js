// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `yield` is a valid expression within generator function bodies.
features: [generators]
es6id: 14.4
---*/

var iter, result;
class A {
  *g1() { (yield 1) }
  *g2() { [yield 1] }
  *g3() { {yield 1} }
  *g4() { yield 1, yield 2; }
  *g5() { (yield 1) ? yield 2 : yield 3; }
}

iter = A.prototype.g1();
result = iter.next();
assert.sameValue(result.value, 1, 'Within grouping operator: result `value`');
assert.sameValue(
  result.done, false, 'Within grouping operator: result `done` flag'
);
result = iter.next();
assert.sameValue(
  result.value, undefined, 'Following grouping operator: result `value`'
);
assert.sameValue(
  result.done, true, 'Following grouping operator: result `done` flag'
);

iter = A.prototype.g2();
result = iter.next();
assert.sameValue(result.value, 1, 'Within array literal: result `value`');
assert.sameValue(
  result.done, false, 'Within array literal: result `done` flag'
);
result = iter.next();
assert.sameValue(
  result.value, undefined, 'Following array literal: result `value`'
);
assert.sameValue(
  result.done, true, 'Following array literal: result `done` flag'
);

iter = A.prototype.g3();
result = iter.next();
assert.sameValue(result.value, 1, 'Within object literal: result `value`');
assert.sameValue(
  result.done, false, 'Within object literal: result `done` flag'
);
result = iter.next();
assert.sameValue(
  result.value, undefined, 'Following object literal: result `value`'
);
assert.sameValue(
  result.done, true, 'Following object literal: result `done` flag'
);

iter = A.prototype.g4();
result = iter.next();
assert.sameValue(
  result.value, 1, 'First expression in comma expression: result `value`'
);
assert.sameValue(
  result.done,
  false,
  'First expression in comma expression: result `done` flag'
);
result = iter.next();
assert.sameValue(
  result.value, 2, 'Second expression in comma expression: result `value`'
);
assert.sameValue(
  result.done,
  false,
  'Second expression in comma expression: result `done` flag'
);
result = iter.next();
assert.sameValue(
  result.value, undefined, 'Following comma expression: result `value`'
);
assert.sameValue(
  result.done, true, 'Following comma expression: result `done` flag'
);

iter = A.prototype.g5();
result = iter.next();
assert.sameValue(
  result.value,
  1,
  'Conditional expression in conditional operator: result `value`'
);
assert.sameValue(
  result.done,
  false,
  'Conditional expression in conditional operator: result `done` flag'
);
result = iter.next();
assert.sameValue(
  result.value,
  3,
  'Branch in conditional operator: result `value`'
);
assert.sameValue(
  result.done,
  false,
  'Branch in conditional operator: result `done` flag'
);
result = iter.next();
assert.sameValue(
  result.value, undefined, 'Following conditional operator: result `value`'
);
assert.sameValue(
  result.done, true, 'Following conditional operator: result `done` flag'
);
