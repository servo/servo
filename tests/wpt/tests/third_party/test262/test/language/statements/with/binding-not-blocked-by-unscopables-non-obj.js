// Copyright 2015 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 8.1.1.2.1
description: Non-object values of `Symbol.unscopables` property are ignored
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

var test262ToString = {};
var env = { toString: test262ToString };
env[Symbol.unscopables] = '';

with (env) {
  assert.sameValue(toString, test262ToString);
}
