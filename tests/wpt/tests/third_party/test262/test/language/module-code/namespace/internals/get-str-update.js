// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-get-p-receiver
description: References observe the mutation of initialized bindings
info: |
    [...]
    12. Let targetEnvRec be targetEnv's EnvironmentRecord.
    13. Return ? targetEnvRec.GetBindingValue(binding.[[BindingName]], true).
flags: [module]
---*/

import * as ns from './get-str-update.js';
export var local1 = 111;
var local2 = 222;
export { local2 as renamed };
export { local1 as indirect } from './get-str-update.js';

local1 = 333;
local2 = 444;

assert.sameValue(ns.local1, 333);
assert.sameValue(ns.renamed, 444);
assert.sameValue(ns.indirect, 333);
