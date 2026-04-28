// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-hasproperty-p
description: >
    Behavior of the [[HasProperty]] internal method with a string argument for
    exported uninitialized bindings.
info: |
    [...]
    2. Let exports be the value of O's [[Exports]] internal slot.
    3. If P is an element of exports, return true.
    4. Return false.
flags: [module]
features: [Reflect, let]
---*/

import * as ns from './has-property-str-found-uninit.js';

assert('local1' in ns, 'in: local1');
assert(Reflect.has(ns, 'local1'), 'Reflect.has: local1');

assert('renamed' in ns, 'in: renamed');
assert(Reflect.has(ns, 'renamed'), 'Reflect.has: renamed');

assert('indirect' in ns, 'in: indirect');
assert(Reflect.has(ns, 'indirect'), 'Reflect.has: indirect');

assert('default' in ns, 'in: default');
assert(Reflect.has(ns, 'default'), 'Reflect.has: default');

export let local1 = 23;
let local2 = 45;
export { local2 as renamed };
export { local1 as indirect } from './has-property-str-found-uninit.js';
export default null;
