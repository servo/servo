// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/assertThrowsValue.js, sm/non262-generators-shell.js]
description: |
  pending
esid: pending
---*/
// Test yield* with iter.throw and monkeypatching.

function* g1() { return (yield 1); }
function* g2() { try { yield 1; } catch (e) { yield e; } }
function* delegate(iter) { return yield* iter; }
var GeneratorObjectPrototype = Object.getPrototypeOf(g1).prototype;
var GeneratorObjectPrototype_throw = GeneratorObjectPrototype.throw;

// An uncaught delegated throw.
var inner = g1();
var outer = delegate(inner);
assertIteratorNext(outer, 1);
assertThrowsValue(function () { outer.throw(42) }, 42);
assertThrowsValue(function () { outer.throw(42) }, 42);

// A caught delegated throw.
inner = g2();
outer = delegate(inner);
assertIteratorNext(outer, 1);
assertIteratorResult(outer.throw(42), 42, false);
assertThrowsValue(function () { outer.throw(42) }, 42);
assertThrowsValue(function () { outer.throw(42) }, 42);

// What would be an uncaught delegated throw, but with a monkeypatched iterator.
inner = g1();
outer = delegate(inner);
assertIteratorNext(outer, 1);
inner.throw = function(e) { return { value: e*2 }; };
assert.sameValue(84, outer.throw(42).value);
assertIteratorDone(outer, undefined);

// Monkeypatching inner.next.
inner = g1();
outer = delegate(inner);
inner.next = function() { return { value: 13, done: true } };
assertIteratorDone(outer, 13);

// What would be a caught delegated throw, but with a monkeypunched prototype.
inner = g2();
outer = delegate(inner);
assertIteratorNext(outer, 1);
delete GeneratorObjectPrototype.throw;
var outer_throw_42 = GeneratorObjectPrototype_throw.bind(outer, 42);
// yield* protocol violation: no 'throw' method
assert.throws(TypeError, outer_throw_42);
// Now done, so just throws.
assertThrowsValue(outer_throw_42, 42);

// Monkeypunch a different throw handler.
inner = g2();
outer = delegate(inner);
outer_throw_42 = GeneratorObjectPrototype_throw.bind(outer, 42);
assertIteratorNext(outer, 1);
GeneratorObjectPrototype.throw = function(e) { return { value: e*2 }; }
assert.sameValue(84, outer_throw_42().value);
assert.sameValue(84, outer_throw_42().value);
// This continues indefinitely.
assert.sameValue(84, outer_throw_42().value);
assertIteratorDone(outer, undefined);

// The same, but restoring the original pre-monkey throw.
inner = g2();
outer = delegate(inner);
outer_throw_42 = GeneratorObjectPrototype_throw.bind(outer, 42);
assertIteratorNext(outer, 1);
assert.sameValue(84, outer_throw_42().value);
assert.sameValue(84, outer_throw_42().value);
GeneratorObjectPrototype.throw = GeneratorObjectPrototype_throw;
assertIteratorResult(outer_throw_42(), 42, false);
assertIteratorDone(outer, undefined);

