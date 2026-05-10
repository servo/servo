// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.3.1.3
description: >
    When a generator is paused within a `try` block of a `try..catch` statement
    and before a nested `try..catch` statement, `return` should interrupt
    control flow as if a `return` statement had appeared at that location in
    the function body.
features: [generators]
---*/

var inTry = 0;
var inFinally = 0;
var unreachable = 0;
function* g() {
  try {
    inTry += 1;
    yield;
    try {
      unreachable += 1;
    } catch (e) {
      throw e;
    }
    unreachable += 1;
  } finally {
    inFinally += 1;
  }
  unreachable += 1;
}
var iter = g();
var result;

iter.next();

assert.sameValue(inTry, 1, '`try` code path executed');
assert.sameValue(inFinally, 0, '`finally` code path not executed');

result = iter.return(45);
assert.sameValue(result.value, 45, 'Second result `value`');
assert.sameValue(result.done, true, 'Second result `done` flag');
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
  result.done, true, 'Result `done` flag is `true` when complete'
);
assert.sameValue(
  unreachable, 0, 'statement following `yield` not executed (once "completed")'
);
