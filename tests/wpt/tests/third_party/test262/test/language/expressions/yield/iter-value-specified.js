// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    When the `next` method of a generator-produced iterable is invoked without
    an argument, the corresponding `yield` expression should be evaluated as
    `undefined`.
features: [generators]
---*/

function* g() { actual = yield; }
var expected = {};
var iter = g();
var actual, result;

result = iter.next();
assert.sameValue(result.value, undefined, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');
assert.sameValue(
  actual, undefined, 'Value of `yield` expression (prior to continuation)'
);

result = iter.next(expected);
assert.sameValue(result.value, undefined, 'Second result `value`');
assert.sameValue(result.done, true, 'Second result `done` flag');
assert.sameValue(
  actual, expected, 'Value of `yield` expression (following continuation)'
);
