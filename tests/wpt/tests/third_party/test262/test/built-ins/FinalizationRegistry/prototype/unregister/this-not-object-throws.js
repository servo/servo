// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.unregister
description: Throws a TypeError if this is not an Object
info: |
  FinalizationRegistry.prototype.unregister ( unregisterToken )

  1. Let finalizationRegistry be the this value.
  2. If Type(finalizationRegistry) is not Object, throw a TypeError exception.
  3. If finalizationRegistry does not have a [[Cells]] internal slot, throw a TypeError exception.
  4. If Type(unregisterToken) is not Object, throw a TypeError exception.
  ...
features: [FinalizationRegistry]
---*/

assert.sameValue(typeof FinalizationRegistry.prototype.unregister, 'function');

var unregister = FinalizationRegistry.prototype.unregister;

assert.throws(TypeError, function() {
  unregister.call(undefined, {});
}, 'undefined');

assert.throws(TypeError, function() {
  unregister.call(null, {});
}, 'null');

assert.throws(TypeError, function() {
  unregister.call(true, {});
}, 'true');

assert.throws(TypeError, function() {
  unregister.call(false, {});
}, 'false');

assert.throws(TypeError, function() {
  unregister.call(1, {});
}, 'number');

assert.throws(TypeError, function() {
  unregister.call('object', {});
}, 'string');

var s = Symbol();
assert.throws(TypeError, function() {
  unregister.call(s, {});
}, 'symbol');
