// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `yield` is a valid BindingIdentifier for GeneratorDeclarations outside of
    strict mode.
es6id: 12.1.1
flags: [noStrict]
features: [generators]
---*/

var iter, result;
function* yield() { (yield 3) + (yield 4); }

iter = yield();

result = iter.next();
assert.sameValue(result.value, 3, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iter.next();
assert.sameValue(result.value, 4, 'Second result `value`');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iter.next();
assert.sameValue(result.value, undefined, 'Third result `value`');
assert.sameValue(result.done, true, 'Third result `done` flag');
