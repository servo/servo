// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-generators-shell.js]
description: |
  pending
esid: pending
---*/
// Test yield* with iter.next and monkeypatching.

function* g(n) { for (var i=0; i<n; i++) yield i; }
function* delegate(iter) { return yield* iter; }
var GeneratorObjectPrototype = Object.getPrototypeOf(g).prototype;
var GeneratorObjectPrototype_next = GeneratorObjectPrototype.next;

// Monkeypatch next on an iterator.
var inner = g(20);
var outer = delegate(inner);
var innerNextFn = GeneratorObjectPrototype_next;
inner.next = function() { return innerNextFn.call(this); };
assertIteratorNext(outer, 0);
assertIteratorNext(outer, 1);
innerNextFn = function() { return { patched: true }; };
// 42 yielded directly without re-boxing.
assert.sameValue(true, outer.next().patched);
// Outer generator not terminated.
assert.sameValue(true, outer.next().patched);
// Restore.
innerNextFn = GeneratorObjectPrototype_next;
assertIteratorNext(outer, 2);
// Repatch.
innerNextFn = function() { return { value: 42, done: true }; };
assertIteratorDone(outer, 42);

// Monkeypunch next on the prototype.
var inner = g(20);
var outer = delegate(inner);
var innerNextFn = GeneratorObjectPrototype_next;
GeneratorObjectPrototype.next = function() { return innerNextFn.call(this); };
assertIteratorNext(outer, 0);
assertIteratorNext(outer, 1);
innerNextFn = function() { return { patched: true }; };
// 42 yielded directly without re-boxing.
assert.sameValue(true, GeneratorObjectPrototype_next.call(outer).patched);
// Outer generator not terminated.
assert.sameValue(true, GeneratorObjectPrototype_next.call(outer).patched);
// Restore.
innerNextFn = GeneratorObjectPrototype_next;
assertIteratorNext(outer, 2);

