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
var iter = g();
var actual, result;

result = iter.next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, false);
assert.sameValue(actual, undefined);

result = iter.next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
assert.sameValue(actual, undefined);
