// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `yield` is a valid statement within generator function bodies.
features: [generators]
es6id: 14.4
---*/

var iter, result;
var obj = {
  *g1() { yield; },
  *g2() { yield 1; }
};

iter = obj.g1();
result = iter.next();
assert.sameValue(
  result.value, undefined, 'Without right-hand-side: first result `value`'
);
assert.sameValue(
  result.done, false, 'Without right-hand-side: first result `done` flag'
);
result = iter.next();
assert.sameValue(
  result.value, undefined, 'Without right-hand-side: second result `value`'
);
assert.sameValue(
  result.done, true, 'Without right-hand-eside: second result `done` flag'
);

iter = obj.g2();
result = iter.next();
assert.sameValue(
  result.value, 1, 'With right-hand-side: first result `value`'
);
assert.sameValue(
  result.done, false, 'With right-hand-side: first result `done` flag'
);
result = iter.next();
assert.sameValue(
  result.value, undefined, 'With right-hand-side: second result `value`'
);
assert.sameValue(
  result.done, true, 'With right-hand-eside: second result `done` flag'
);
