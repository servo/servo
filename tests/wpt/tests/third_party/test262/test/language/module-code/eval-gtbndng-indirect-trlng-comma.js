// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    NamedImports in ImportDeclaration may contain a trailing comma
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
flags: [module]
---*/

import { x , } from './eval-gtbndng-indirect-trlng-comma_FIXTURE.js';

assert.sameValue(x, 1);
