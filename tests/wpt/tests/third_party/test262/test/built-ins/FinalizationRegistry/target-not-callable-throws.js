// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry-target
description: >
  Throws a TypeError if target is not callable
info: |
  FinalizationRegistry ( cleanupCallback )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. If IsCallable(cleanupCallback) is false, throw a TypeError exception.
  ...
features: [FinalizationRegistry, WeakRef]
---*/

assert.sameValue(
  typeof FinalizationRegistry, 'function',
  'typeof FinalizationRegistry is function'
);

assert.throws(TypeError, function() {
  new FinalizationRegistry({});
}, 'ordinary object');

assert.throws(TypeError, function() {
  new FinalizationRegistry(WeakRef.prototype);
}, 'WeakRef.prototype');

assert.throws(TypeError, function() {
  new FinalizationRegistry(FinalizationRegistry.prototype);
}, 'FinalizationRegistry.prototype');

assert.throws(TypeError, function() {
  new FinalizationRegistry([]);
}, 'Array');

assert.throws(TypeError, function() {
  new FinalizationRegistry();
}, 'implicit undefined');

assert.throws(TypeError, function() {
  new FinalizationRegistry(undefined);
}, 'explicit undefined');

assert.throws(TypeError, function() {
  new FinalizationRegistry(null);
}, 'null');

assert.throws(TypeError, function() {
  new FinalizationRegistry(1);
}, 'number');

assert.throws(TypeError, function() {
  new FinalizationRegistry('Object');
}, 'string');

var s = Symbol();
assert.throws(TypeError, function() {
  new FinalizationRegistry(s);
}, 'symbol');

assert.throws(TypeError, function() {
  new FinalizationRegistry(true);
}, 'Boolean, true');

assert.throws(TypeError, function() {
  new FinalizationRegistry(false);
}, 'Boolean, false');
