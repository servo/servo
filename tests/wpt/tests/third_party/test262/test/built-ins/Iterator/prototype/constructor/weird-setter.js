// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set-iteratorprototype-constructor
description: weird setter
info: |
  The value of the [[Get]] attribute is a built-in function that requires no arguments. It performs the following steps when called:
    1. Return %Iterator%.

  The value of the [[Set]] attribute is a built-in function that takes an argument _v_. It performs the following steps when called:
    1. Perform ? SetterThatIgnoresPrototypeProperties(%Iterator.prototype%, *"constructor"*, _v_).
    2. Return *undefined*.
features: [iterator-helpers]
---*/

let IteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf([][Symbol.iterator]()))

let sentinel = {};

let { get, set } = Object.getOwnPropertyDescriptor(Iterator.prototype, 'constructor');

assert.sameValue(Iterator.prototype.constructor, Iterator);
assert.sameValue(get.call(), Iterator);

// 1. If _this_ is not an Object, then
//   1. Throw a *TypeError* exception.
assert.throws(TypeError, () => set.call(undefined, ''));
assert.throws(TypeError, () => set.call(null, ''));
assert.throws(TypeError, () => set.call(true, ''));

// 1. If _this_ is _home_, then
//   1. NOTE: Throwing here emulates assignment to a non-writable data property on the _home_ object in strict mode code.
//   1. Throw a *TypeError* exception.
assert.throws(TypeError, () => set.call(IteratorPrototype, ''));
assert.throws(TypeError, () => IteratorPrototype.constructor = '');

assert.sameValue(Iterator.prototype.constructor, Iterator);
assert.sameValue(get.call(), Iterator);

// 1. If _desc_ is *undefined*, then
//   1. Perform ? CreateDataPropertyOrThrow(_this_, _p_, _v_).
let FakeGeneratorPrototype = Object.create(IteratorPrototype);
Object.freeze(IteratorPrototype);
FakeGeneratorPrototype.constructor = sentinel;
assert.sameValue(FakeGeneratorPrototype.constructor, sentinel);

assert.sameValue(Iterator.prototype.constructor, Iterator);
assert.sameValue(get.call(), Iterator);

// 1. Else,
//   1. Perform ? Set(_this_, _p_, _v_, *true*).
let o = { constructor: sentinel + 'a' };
set.call(o, sentinel);
assert.sameValue(o.constructor, sentinel);

assert.sameValue(Iterator.prototype.constructor, Iterator);
assert.sameValue(get.call(), Iterator);
