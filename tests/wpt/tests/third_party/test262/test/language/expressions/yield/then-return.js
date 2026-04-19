// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    When a generator body contains a yield statement followed by a return
    statement, it should produce an iterator that visits the yieled value and
    completes on the returned value.
features: [generators]
---*/

function* g() { yield 1; return 2; }
var iter = g();
var result;

result = iter.next();
assert.sameValue(result.value, 1, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iter.next();
assert.sameValue(result.value, 2, 'Second result `value`');
assert.sameValue(result.done, true, 'Second result `done` flag');
