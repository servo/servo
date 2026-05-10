// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.3.1.4
description: >
    When a generator is paused within the `finally` block of a `try..finally`
    statement, `throw` should interrupt control flow as if a `throw` statement
    had appeared at that location in the function body.
features: [generators]
---*/

var unreachable = 0;

function* g() {
  yield 1;
  try {
    yield 2;
  } finally {
    yield 3;
    unreachable += 1;
  }
  yield 4;
}
var iter = g();
var result;

result = iter.next();
assert.sameValue(result.value, 1, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iter.next();
assert.sameValue(result.value, 2, 'Second result `value`');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iter.next();
assert.sameValue(result.value, 3, 'Third result `value`');
assert.sameValue(result.done, false, 'Third result `done` flag');

assert.throws(Test262Error, function() {
  iter.throw(new Test262Error());
});

assert.sameValue(
  unreachable,
  0,
  'statement following `yield` not executed (following `throw`)'
);

result = iter.next();
assert.sameValue(
  result.value, undefined, 'Result `value` is undefined when done'
);
assert.sameValue(result.done, true, 'Result `done` flag is `true` when done');
assert.sameValue(
  unreachable, 0, 'statement following `yield` not executed (once "completed")'
);

iter.next();
