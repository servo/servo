// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    When invoked via the function invocation pattern with multiple arguments,
    the GeneratorFunction intrinsic creates a valid generator whose body is the
    last argument evaluated as source code and whose formal parameters are
    defined by the preceding arguments.
features: [generators]
---*/

var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;

var g = GeneratorFunction('x', 'y', 'yield x + y;');
var iter = g(2, 3);
var result;

result = iter.next();
assert.sameValue(result.value, 5, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iter.next();
assert.sameValue(result.value, undefined, 'Final result `value`');
assert.sameValue(result.done, true, 'Final result `done` flag');
