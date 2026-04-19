// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.3.1.3
description: >
    When a generator is paused within a `catch` block that is declared within a
    `try` block of a `try..catch` statement, `return` should interrupt control
    flow as if a `return` statement had appeared at that location in the
    function body.
features: [generators]
---*/

var inCatch = 0;
var inFinally = 0;
var unreachable = 0;
function* g() {
  try {
    try {
      throw new Error();
    } catch (e) {
      inCatch += 1;
      yield;
      unreachable += 1;
    }
    unreachable += 1;
  } finally {
    inFinally += 1;
  }
  unreachable += 1;
}
var iter = g();
var result;

result = iter.next();

assert.sameValue(inCatch, 1, '`catch` code patch executed');
assert.sameValue(inFinally, 0, '`finally` code path not executed');

result = iter.return(45);
assert.sameValue(result.value, 45, 'Result `value` following `return`');
assert.sameValue(result.done, true, 'Result `done` flag following `return`');
assert.sameValue(inFinally, 1, '`finally` code path executed');
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
  result.done, true, 'Result `done` flag is `true` when compelete'
);
assert.sameValue(
  unreachable, 0, 'statement following `yield` not executed (once "completed")'
);
