// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-getownproperty-p
description: >
    Behavior of the [[GetOwnProperty]] internal method with a string argument
    describing an uninitialized binding
info: |
    1. If Type(P) is Symbol, return OrdinaryGetOwnProperty(O, P).
    2. Let exports be the value of O's [[Exports]] internal slot.
    3. If P is not an element of exports, return undefined.
    4. Let value be ? O.[[Get]](P, O).
flags: [module]
features: [let]
---*/

import * as ns from './get-own-property-str-found-uninit.js';

assert.throws(ReferenceError, function() {
  Object.prototype.hasOwnProperty.call(ns, 'local1');
}, 'hasOwnProperty: local1');
assert.throws(ReferenceError, function() {
  Object.getOwnPropertyDescriptor(ns, 'local1');
}, 'getOwnPropertyDescriptor: local1');

assert.throws(ReferenceError, function() {
  Object.prototype.hasOwnProperty.call(ns, 'renamed');
}, 'hasOwnProperty: renamed');
assert.throws(ReferenceError, function() {
  Object.getOwnPropertyDescriptor(ns, 'renamed');
}, 'getOwnPropertyDescriptor: renamed');

assert.throws(ReferenceError, function() {
  Object.prototype.hasOwnProperty.call(ns, 'indirect');
}, 'hasOwnProperty: indirect');
assert.throws(ReferenceError, function() {
  Object.getOwnPropertyDescriptor(ns, 'indirect');
}, 'getOwnPropertyDescriptor: indirect');

assert.throws(ReferenceError, function() {
  Object.prototype.hasOwnProperty.call(ns, 'default');
}, 'hasOwnProperty: default');
assert.throws(ReferenceError, function() {
  Object.getOwnPropertyDescriptor(ns, 'default');
}, 'getOwnPropertyDescriptor: default');

export let local1 = 23;
let local2 = 45;
export { local2 as renamed };
export { local1 as indirect } from './get-own-property-str-found-uninit.js';
export default null;
