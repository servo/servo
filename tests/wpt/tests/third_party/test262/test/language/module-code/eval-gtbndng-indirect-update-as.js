// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Modifications to named bindings that occur after dependency has been
    evaluated are reflected in local binding
esid: sec-moduleevaluation
info: |
    8.1.1.5.1 GetBindingValue (N, S)

    [...]
    3. If the binding for N is an indirect binding, then
       a. Let M and N2 be the indirection values provided when this binding for
          N was created.
       b. Let targetEnv be M.[[Environment]].
       c. If targetEnv is undefined, throw a ReferenceError exception.
       d. Let targetER be targetEnv's EnvironmentRecord.
       e. Return ? targetER.GetBindingValue(N2, S).
includes: [fnGlobalObject.js]
flags: [module]
---*/

import { x as y, x as z } from './eval-gtbndng-indirect-update-as_FIXTURE.js';

assert.sameValue(y, 1);
assert.sameValue(z, 1);

// This function is exposed on the global scope (instead of as an exported
// binding) in order to avoid possible false positives from assuming correct
// behavior of the semantics under test.
fnGlobalObject().test262update();

assert.sameValue(y, 2);
assert.sameValue(z, 2);
