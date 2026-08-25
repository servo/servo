// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap-iterable
description: >
  Throws a TypeError if keys in iterable items cannot be held weakly.
info: |
  WeakMap ( [ _iterable_ ] )
  5. Let _adder_ be ? Get(_map_, *"set"*).
  6. Return ? AddEntriesFromIterable(_map_, _iterable_, _adder_).

  AddEntriesFromIterable:
  3. Repeat,
    i. Let _status_ be Completion(Call(_adder_, _target_, « _k_, _v_ »)).
    j. IfAbruptCloseIterator(_status_, _iteratorRecord_).

  WeakMap.prototype.set( _key_, _value_ ):
  4. If CanBeHeldWeakly(_key_) is *false*, throw a *TypeError* exception.
features: [Symbol, WeakMap]
---*/

assert.throws(TypeError, function() {
  new WeakMap([1, 1]);
});

assert.throws(TypeError, function() {
  new WeakMap(['', 1]);
});

assert.throws(TypeError, function() {
  new WeakMap([true, 1]);
});

assert.throws(TypeError, function() {
  new WeakMap([null, 1]);
});

assert.throws(TypeError, function() {
  new WeakMap([Symbol.for('registered symbol'), 1]);
}, 'Registered symbol not allowed as a WeakMap key');

assert.throws(TypeError, function() {
  new WeakMap([undefined, 1]);
});

assert.throws(TypeError, function() {
  new WeakMap([
    ['a', 1], 2
  ]);
});
