// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref.prototype.deref
description: Throws a TypeError if this is not an Object
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
features: [WeakRef]
---*/

assert.sameValue(typeof WeakRef.prototype.deref, 'function');

var deref = WeakRef.prototype.deref;

assert.throws(TypeError, function() {
  deref.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  deref.call(null);
}, 'null');

assert.throws(TypeError, function() {
  deref.call(true);
}, 'true');

assert.throws(TypeError, function() {
  deref.call(false);
}, 'false');

assert.throws(TypeError, function() {
  deref.call(1);
}, 'number');

assert.throws(TypeError, function() {
  deref.call('object');
}, 'string');

var s = Symbol();
assert.throws(TypeError, function() {
  deref.call(s);
}, 'symbol');
