// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `yield` may be used as the binding identifier of a function expression
    within generator bodies.
es6id: 14.1
flags: [noStrict]
features: [generators]
---*/

var result;
function* g() {
  (function yield() {})
}

result = g().next();

assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
