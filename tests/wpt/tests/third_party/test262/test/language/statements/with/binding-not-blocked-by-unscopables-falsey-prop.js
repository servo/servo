// Copyright 2015 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 8.1.1.2.1
description: >
    False-coercing `Symbol.unscopables` properties do not block access to object environment record
info: |
    [...]
    6. If the withEnvironment flag of envRec is false, return true.
    7. Let unscopables be Get(bindings, @@unscopables).
    8. ReturnIfAbrupt(unscopables).
    9. If Type(unscopables) is Object, then
       a. Let blocked be ToBoolean(Get(unscopables, N)).
       b. ReturnIfAbrupt(blocked).
       c. If blocked is true, return false.
    10. Return true.

    ES6: 13.11.7 (The `with` Statement) Runtime Semantics: Evaluation
    [...]
    6. Set the withEnvironment flag of newEnv’s EnvironmentRecord to true.
    [...]
flags: [noStrict]
features: [Symbol.unscopables]
---*/

var x = 0;
var env = { x: 1 };
env[Symbol.unscopables] = {};

with (env) {
  assert.sameValue(x, 1, 'undefined (no property defined)');
}

env[Symbol.unscopables].x = false;
with (env) {
  assert.sameValue(x, 1, 'literal `false` value');
}

env[Symbol.unscopables].x = undefined;
with (env) {
  assert.sameValue(x, 1, 'literal `undefined` value');
}

env[Symbol.unscopables].x = null;
with (env) {
  assert.sameValue(x, 1, 'null value');
}

env[Symbol.unscopables].x = 0;
with (env) {
  assert.sameValue(x, 1, 'literal `0` number value');
}

env[Symbol.unscopables].x = '';
with (env) {
  assert.sameValue(x, 1, 'empty string value');
}
