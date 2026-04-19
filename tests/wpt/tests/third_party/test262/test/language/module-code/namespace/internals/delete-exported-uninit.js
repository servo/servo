// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-delete-p
description: >
    [[Delete]] behavior for a key that describes an uninitialized exported
    binding
info: |
    [...]
    2. Let exports be the value of O's [[Exports]] internal slot.
    3. If P is an element of exports, return false.
flags: [module]
features: [Reflect, let]
---*/

import * as ns from './delete-exported-uninit.js';

assert.throws(TypeError, function() {
  delete ns.local1;
}, 'delete: local1');
assert.sameValue(
  Reflect.deleteProperty(ns, 'local1'), false, 'Reflect.deleteProperty: local1'
);
assert.throws(ReferenceError, function() {
  ns.local1;
}, 'binding unmodified: local1');

assert.throws(TypeError, function() {
  delete ns.renamed;
}, 'delete: renamed');
assert.sameValue(
  Reflect.deleteProperty(ns, 'renamed'), false, 'Reflect.deleteProperty: renamed'
);
assert.throws(ReferenceError, function() {
  ns.renamed;
}, 'binding unmodified: renamed');

assert.throws(TypeError, function() {
  delete ns.indirect;
}, 'delete: indirect');
assert.sameValue(
  Reflect.deleteProperty(ns, 'indirect'),
  false,
  'Reflect.deleteProperty: indirect'
);
assert.throws(ReferenceError, function() {
  ns.indirect;
}, 'binding unmodified: indirect');

assert.throws(TypeError, function() {
  delete ns.default;
}, 'delete: default');
assert.sameValue(
  Reflect.deleteProperty(ns, 'default'),
  false,
  'Reflect.deleteProperty: default'
);
assert.throws(ReferenceError, function() {
  ns.default;
}, 'binding unmodified: default');

export let local1 = 23;
let local2 = 45;
export { local2 as renamed };
export { local1 as indirect } from './delete-exported-uninit.js';
export default null;
