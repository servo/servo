// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.3.1.3
description: >
    When a generator is paused before a `try..finally` statement, `return`
    should interrupt control flow as if a `return` statement had appeared at
    that location in the function body.
features: [generators]
---*/

var unreachable = 0;

function* g() {
  yield;
  unreachable += 1;
  try {} finally {}
}
var iter = g();
var result;

iter.next();
assert.sameValue(
  unreachable,
  0,
  'statement following `yield` not executed (paused at yield)'
);

result = iter.return(45);
assert.sameValue(result.value, 45, 'Result `value` following `return`');
assert.sameValue(result.done, true, 'Result `done` flag following `return`');
assert.sameValue(
  unreachable,
  0,
  'statement following `yield` not executed (following `return`)'
);

result = iter.next();
assert.sameValue(
  result.value, undefined, 'Result `value` is undefined when complete'
);
assert.sameValue(
  result.done, true, 'Result `done` flag is `true` when complete'
);
assert.sameValue(
  unreachable, 0, 'statement following `yield` not executed (once "completed")'
);
