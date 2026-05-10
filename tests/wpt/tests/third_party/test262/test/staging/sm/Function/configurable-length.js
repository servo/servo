/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Very simple initial test that the "length" property of a function is
// configurable. More thorough tests follow.
var f = function (a1, a2, a3, a4) {};
assert.sameValue(delete f.length, true);
assert.sameValue(f.hasOwnProperty("length"), false);
assert.sameValue(f.length, 0);  // inherited from Function.prototype.length
assert.sameValue(delete Function.prototype.length, true);
assert.sameValue(f.length, undefined);


// Now for the details.
//
// Many of these tests are poking at the "resolve hook" mechanism SM uses to
// lazily create this property, which is wonky and deserving of some extra
// skepticism.

// We've deleted Function.prototype.length. Check that the resolve hook does
// not resurrect it.
assert.sameValue("length" in Function.prototype, false);
Function.prototype.length = 7;
assert.sameValue(Function.prototype.length, 7);
delete Function.prototype.length;
assert.sameValue(Function.prototype.length, undefined);

// OK, define Function.prototype.length back to its original state per spec, so
// the remaining tests can run in a more typical environment.
Object.defineProperty(Function.prototype, "length", {value: 0, configurable: true});

// Check the property descriptor of a function length property.
var g = function f(a1, a2, a3, a4, a5) {};
var desc = Object.getOwnPropertyDescriptor(g, "length");
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.writable, false);
assert.sameValue(desc.value, 5);

// After deleting the length property, assigning to f.length fails because
// Function.prototype.length is non-writable. In strict mode it would throw.
delete g.length;
g.length = 12;
assert.sameValue(g.hasOwnProperty("length"), false);
assert.sameValue(g.length, 0);

// After deleting both the length property and Function.prototype.length,
// assigning to f.length creates a new plain old data property.
delete Function.prototype.length;
g.length = 13;
var desc = Object.getOwnPropertyDescriptor(g, "length");
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.writable, true);
assert.sameValue(desc.value, 13);

// Deleting the .length of one instance of a FunctionDeclaration does not
// affect other instances.
function mkfun() {
    function fun(a1, a2, a3, a4, a5) {}
    return fun;
}
g = mkfun();
var h = mkfun();
delete h.length;
assert.sameValue(g.length, 5);
assert.sameValue(mkfun().length, 5);

// Object.defineProperty on a brand-new function is sufficient to cause the
// LENGTH_RESOLVED flag to be set.
g = mkfun();
Object.defineProperty(g, "length", {value: 0});
assert.sameValue(delete g.length, true);
assert.sameValue(g.hasOwnProperty("length"), false);

// Object.defineProperty on a brand-new function correctly merges new
// attributes with the builtin ones.
g = mkfun();
Object.defineProperty(g, "length", { value: 42 });
desc = Object.getOwnPropertyDescriptor(g, "length");
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.writable, false);
assert.sameValue(desc.value, 42);

