// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-get-p-receiver
description: >
    Behavior of the [[Get]] internal method with a symbol argument that can be
    found
info: |
    [...]
    2. If Type(P) is Symbol, then
       a. Return ? OrdinaryGet(O, P, Receiver).
flags: [module]
features: [Symbol.toStringTag]
---*/

import * as ns from './get-sym-found.js';

assert.sameValue(typeof ns[Symbol.toStringTag], 'string');
