// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `yield` is a valid expression within generator function bodies.
features: [generators]
es6id: 14.4
---*/

var iter, result;
var obj = {
  *g1() { (yield) },
  *g2() { [yield] },
  *g3() { {yield} },
  *g4() { yield, yield; },
  *g5() { (yield) ? yield : yield; }
};

iter = obj.g1();
result = iter.next();
assert.sameValue(
  result.value, undefined, 'Within grouping operator: result `value`'
);
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

iter = obj.g2();
result = iter.next();
assert.sameValue(
  result.value, undefined, 'Within array literal: result `value`'
);
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

iter = obj.g3();
result = iter.next();
assert.sameValue(
  result.value, undefined, 'Within object literal: result `value`'
);
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

iter = obj.g4();
result = iter.next();
assert.sameValue(
  result.value,
  undefined,
  'First expression in comma expression: result `value`'
);
assert.sameValue(
  result.done,
  false,
  'First expression in comma expression: result `done` flag'
);
result = iter.next();
assert.sameValue(
  result.value,
  undefined,
  'Second expression in comma expression: result `value`'
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

iter = obj.g5();
result = iter.next();
assert.sameValue(
  result.value,
  undefined,
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
  undefined,
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
