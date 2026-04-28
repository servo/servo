// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.register
description: Throws a TypeError if this is not an Object
info: |
  FinalizationRegistry.prototype.register ( target , holdings [, unregisterToken ] )

  1. Let finalizationRegistry be the this value.
  2. If Type(finalizationRegistry) is not Object, throw a TypeError exception.
  3. If Type(target) is not Object, throw a TypeError exception.
  4. If finalizationRegistry does not have a [[Cells]] internal slot, throw a TypeError exception.
  5. If Type(unregisterToken) is not Object,
    a. If unregisterToken is not undefined, throw a TypeError exception.
  ...
features: [FinalizationRegistry]
---*/

assert.sameValue(typeof FinalizationRegistry.prototype.register, 'function');

var register = FinalizationRegistry.prototype.register;

assert.throws(TypeError, function() {
  register.call(undefined, {});
}, 'undefined');

assert.throws(TypeError, function() {
  register.call(null, {});
}, 'null');

assert.throws(TypeError, function() {
  register.call(true, {});
}, 'true');

assert.throws(TypeError, function() {
  register.call(false, {});
}, 'false');

assert.throws(TypeError, function() {
  register.call(1, {});
}, 'number');

assert.throws(TypeError, function() {
  register.call('object', {});
}, 'string');

var s = Symbol();
assert.throws(TypeError, function() {
  register.call(s, {});
}, 'symbol');
