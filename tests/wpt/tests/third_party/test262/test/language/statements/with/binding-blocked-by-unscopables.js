// Copyright 2015 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 8.1.1.2.1
description: >
    True-coercing `Symbol.unscopables` properties block access to object environment record
info: |
    [...]
    6. If the withEnvironment flag of envRec is false, return true.
    7. Let unscopables be Get(bindings, @@unscopables).
    8. ReturnIfAbrupt(unscopables).
    9. If Type(unscopables) is Object, then
       a. Let blocked be ToBoolean(Get(unscopables, N)).
       b. ReturnIfAbrupt(blocked).
       c. If blocked is true, return false.

    ES6: 13.11.7 (The `with` Statement) Runtime Semantics: Evaluation
    [...]
    6. Set the withEnvironment flag of newEnv’s EnvironmentRecord to true.
    [...]
flags: [noStrict]
features: [Symbol.unscopables]
---*/

var x = 0;
var env = { x: 1 };
env[Symbol.unscopables] = { x: true };

with (env) {
  assert.sameValue(x, 0, 'literal `true` value');
}

env[Symbol.unscopables].x = 'string';
with (env) {
  assert.sameValue(x, 0, 'non-empty string values');
}

env[Symbol.unscopables].x = 86;
with (env) {
  assert.sameValue(x, 0, 'non-zero number values');
}

env[Symbol.unscopables].x = {};
with (env) {
  assert.sameValue(x, 0, 'object values');
}

env[Symbol.unscopables].x = Symbol.unscopables;
with (env) {
  assert.sameValue(x, 0, 'Symbol values');
}
