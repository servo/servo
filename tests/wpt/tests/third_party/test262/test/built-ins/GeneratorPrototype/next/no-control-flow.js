// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    When a generator body contains no control flow statements, it should
    produce an iterator that is initially completed with `undefined` as its
    value.
features: [generators]
---*/

function* g() {}
var iter = g();
var result;

result = iter.next();
assert.sameValue(result.value, undefined, 'First result `value`');
assert.sameValue(result.done, true, 'Second result `done` flag');

result = iter.next();
assert.sameValue(result.value, undefined, 'Second result `value`');
assert.sameValue(result.done, true, 'Second result `done` flag');
