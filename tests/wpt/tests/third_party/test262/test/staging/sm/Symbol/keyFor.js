/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
assert.sameValue(Symbol.keyFor(Symbol.for("moon")), "moon");
assert.sameValue(Symbol.keyFor(Symbol.for("")), "");
assert.sameValue(Symbol.keyFor(Symbol("moon")), undefined);
assert.sameValue(Symbol.keyFor(Symbol.iterator), undefined);

assert.throws(TypeError, () => Symbol.keyFor());
assert.throws(TypeError, () => Symbol.keyFor(Object(Symbol("moon"))));

assert.sameValue(Symbol.keyFor.length, 1);

