// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.3.1.3
description: >
    Resuming abruptly from a generator in the 'completed' state should honor the
    abrupt completion and remain in the 'completed' state.
features: [generators]
---*/

function* G() {}
var iter, result;

iter = G();
iter.next();

result = iter.return(33);

assert.sameValue(result.value, 33, 'return: result `value`');
assert.sameValue(result.done, true, 'return: result `done` flag');

result = iter.next();

assert.sameValue(result.value, undefined, 'next: result `value`');
assert.sameValue(result.done, true, 'next: result `done` flag');
