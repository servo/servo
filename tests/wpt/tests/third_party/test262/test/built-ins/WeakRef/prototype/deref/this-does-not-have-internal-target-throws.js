// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref.prototype.deref
description: Throws a TypeError if this does not have a [[Target]] internal slot
info: |
  WeakRef.prototype.deref ()

  1. Let weakRef be the this value.
  2. If Type(weakRef) is not Object, throw a TypeError exception.
  3. If weakRef does not have a [[Target]] internal slot, throw a TypeError exception.
  4. Let target be the value of weakRef.[[Target]].
  5. If target is not empty,
    a. Perform ! KeepDuringJob(target).
    b. Return target.
  6. Return undefined.
features: [WeakSet, WeakMap, WeakRef, FinalizationRegistry]
---*/

assert.sameValue(typeof WeakRef.prototype.deref, 'function');

var deref = WeakRef.prototype.deref;

assert.throws(TypeError, function() {
  deref.call({ ['[[Target]]']: {} });
}, 'Ordinary object without [[Target]]');

assert.throws(TypeError, function() {
  deref.call(WeakRef.prototype);
}, 'WeakRef.prototype does not have a [[Target]] internal slot');

assert.throws(TypeError, function() {
  deref.call(WeakRef);
}, 'WeakRef does not have a [[Target]] internal slot');

var finalizationRegistry = new FinalizationRegistry(function() {});
assert.throws(TypeError, function() {
  deref.call(finalizationRegistry);
}, 'FinalizationRegistry instance');

var wm = new WeakMap();
assert.throws(TypeError, function() {
  deref.call(wm);
}, 'WeakMap instance');

var ws = new WeakSet();
assert.throws(TypeError, function() {
  deref.call(ws);
}, 'WeakSet instance');
