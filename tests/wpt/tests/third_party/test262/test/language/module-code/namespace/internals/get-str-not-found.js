// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-get-p-receiver
description: >
    Behavior of the [[Get]] internal method with a string argument for
    non-exported bindings
info: |
    [...]
    3. Let exports be the value of O's [[Exports]] internal slot.
    4. If P is not an element of exports, return undefined.
flags: [module]
---*/

import * as ns from './get-str-not-found.js';
var test262;
export { test262 as anotherName };

assert.sameValue(ns.test262, undefined, 'key: test262');
assert.sameValue(ns.toStringTag, undefined, 'key: toStringTag');
assert.sameValue(ns.iterator, undefined, 'key: iterator');
assert.sameValue(ns.__proto__, undefined, 'key: __proto__');
assert.sameValue(ns.default, undefined, 'key: default');
