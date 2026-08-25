// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref-target
description: >
  Throws a TypeError if target cannot be held weakly
info: |
  WeakRef ( _target_ )
  2. If CanBeHeldWeakly(_target_) is *false*, throw a *TypeError* exception.
features: [WeakRef]
---*/

assert.sameValue(
  typeof WeakRef, 'function',
  'typeof WeakRef is function'
);

assert.throws(TypeError, function() {
  new WeakRef();
}, 'implicit undefined');

assert.throws(TypeError, function() {
  new WeakRef(undefined);
}, 'explicit undefined');

assert.throws(TypeError, function() {
  new WeakRef(null);
}, 'null');

assert.throws(TypeError, function() {
  new WeakRef(1);
}, 'number');

assert.throws(TypeError, function() {
  new WeakRef('Object');
}, 'string');

var s = Symbol.for('registered symbol');
assert.throws(TypeError, function() {
  new WeakRef(s);
}, 'registered symbol');

assert.throws(TypeError, function() {
  new WeakRef(true);
}, 'Boolean, true');

assert.throws(TypeError, function() {
  new WeakRef(false);
}, 'Boolean, false');
