// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-set-p-v-receiver
description: The [[Set]] internal method consistently returns `false`
info: |
    1. Return false.
flags: [module]
features: [Reflect, Symbol, Symbol.toStringTag]
---*/

import * as ns from './set.js';
export var local1 = null;
var local2 = null;
export { local2 as renamed };
export { local1 as indirect } from './set.js';
var sym = Symbol('test262');

assert.sameValue(Reflect.set(ns, 'local1'), false, 'Reflect.set: local1');
assert.throws(TypeError, function() {
  ns.local1 = null;
}, 'AssignmentExpression: local1');

assert.sameValue(Reflect.set(ns, 'local2'), false, 'Reflect.set: local2');
assert.throws(TypeError, function() {
  ns.local2 = null;
}, 'AssignmentExpression: local2');

assert.sameValue(Reflect.set(ns, 'renamed'), false, 'Reflect.set: renamed');
assert.throws(TypeError, function() {
  ns.renamed = null;
}, 'AssignmentExpression: renamed');

assert.sameValue(Reflect.set(ns, 'indirect'), false, 'Reflect.set: indirect');
assert.throws(TypeError, function() {
  ns.indirect = null;
}, 'AssignmentExpression: indirect');

assert.sameValue(Reflect.set(ns, 'default'), false, 'Reflect.set: default');
assert.throws(TypeError, function() {
  ns.default = null;
}, 'AssignmentExpression: default');

assert.sameValue(
  Reflect.set(ns, Symbol.toStringTag),
  false,
  'Reflect.set: Symbol.toStringTag'
);
assert.throws(TypeError, function() {
  ns[Symbol.toStringTag] = null;
}, 'AssignmentExpression: Symbol.toStringTag');

assert.sameValue(Reflect.set(ns, sym), false, 'Reflect.set: sym');
assert.throws(TypeError, function() {
  ns[sym] = null;
}, 'AssignmentExpression: sym');
