/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var sym = Symbol.for("hello");
function F() {}
var f = new F();

// inherited data property
F.prototype[sym] = "world";
assert.sameValue(sym in f, true);
assert.sameValue(f.hasOwnProperty(sym), false);
assert.sameValue(f[sym], "world");

// shadowing assignment
f[sym] = "kitty";
assert.sameValue(f[sym], "kitty");
assert.sameValue(F.prototype[sym], "world");

// deletion, revealing previously shadowed property
assert.sameValue(delete f[sym], true);
assert.sameValue(f.hasOwnProperty(sym), false);
assert.sameValue(f[sym], "world");

// inherited accessor property
var value = undefined;
Object.defineProperty(F.prototype, sym, {
    configurable: true,
    get: function () { return 23; },
    set: function (v) { value = v; }
});
assert.sameValue(sym in f, true);
assert.sameValue(f.hasOwnProperty(sym), false);
assert.sameValue(f[sym], 23);
f[sym] = "gravity";
assert.sameValue(value, "gravity");

// inherited accessor property with no setter
Object.defineProperty(F.prototype, sym, {
    set: undefined
});
assert.throws(TypeError, function () { "use strict"; f[sym] = 0; });

// deeply inherited accessor property
var g = Object.create(f);
for (var i = 0; i < 100; i++)
    g = Object.create(g);
assert.sameValue(g[sym], 23);

