// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-get-p-receiver
description: >
    Behavior of the [[Get]] internal method with a string argument for exported
    uninitialized bindings.
info: |
    [...]
    12. Let targetEnvRec be targetEnv's EnvironmentRecord.
    13. Return ? targetEnvRec.GetBindingValue(binding.[[BindingName]], true).
flags: [module]
features: [let]
---*/

import * as ns from './get-str-found-uninit.js';

assert.throws(ReferenceError, function() {
  ns.local1;
});
assert.throws(ReferenceError, function() {
  ns.renamed;
});
assert.throws(ReferenceError, function() {
  ns.indirect;
});
assert.throws(ReferenceError, function() {
  ns.default;
});

export let local1 = 23;
let local2 = 45;
export { local2 as renamed };
export { local1 as indirect } from './get-str-found-uninit.js';
export default null;
