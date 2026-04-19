/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
var m = new Map;

// Symbols can be Map keys.
var sym = Symbol();
m.set(sym, "zero");
assert.sameValue(m.has(sym), true);
assert.sameValue(m.get(sym), "zero");
assert.sameValue(m.has(Symbol()), false);
assert.sameValue(m.get(Symbol()), undefined);
assert.sameValue([...m][0][0], sym);
m.set(sym, "replaced");
assert.sameValue(m.get(sym), "replaced");
m.delete(sym);
assert.sameValue(m.has(sym), false);
assert.sameValue(m.size, 0);

// Symbols returned by Symbol.for() can be Map keys.
for (var word of "that that is is that that is not is not is that not it".split(' ')) {
    sym = Symbol.for(word);
    m.set(sym, (m.get(sym) || 0) + 1);
}
assert.deepEqual([...m], [
    [Symbol.for("that"), 5],
    [Symbol.for("is"), 5],
    [Symbol.for("not"), 3],
    [Symbol.for("it"), 1]
]);

