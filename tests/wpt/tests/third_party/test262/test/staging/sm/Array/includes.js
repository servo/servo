/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Implement Array.prototype.includes
info: bugzilla.mozilla.org/show_bug.cgi?id=1069063
esid: pending
---*/

assert.sameValue(typeof [].includes, "function");
assert.sameValue([].includes.length, 1);

assertTrue([1, 2, 3].includes(2));
assertTrue([1,,2].includes(2));
assertTrue([1, 2, 3].includes(2, 1));
assertTrue([1, 2, 3].includes(2, -2));
assertTrue([1, 2, 3].includes(2, -100));
assertTrue([Object, Function, Array].includes(Function));
assertTrue([-0].includes(0));
assertTrue([NaN].includes(NaN));
assertTrue([,].includes());
assertTrue(staticIncludes("123", "2"));
assertTrue(staticIncludes({length: 3, 1: 2}, 2));
assertTrue(staticIncludes({length: 3, 1: 2, get 3(){throw ""}}, 2));
assertTrue(staticIncludes({length: 3, get 1() {return 2}}, 2));
assertTrue(staticIncludes({__proto__: {1: 2}, length: 3}, 2));
assertTrue(staticIncludes(new Proxy([1], {get(){return 2}}), 2));

assertFalse([1, 2, 3].includes("2"));
assertFalse([1, 2, 3].includes(2, 2));
assertFalse([1, 2, 3].includes(2, -1));
assertFalse([undefined].includes(NaN));
assertFalse([{}].includes({}));
assertFalse(staticIncludes({length: 3, 1: 2}, 2, 2));
assertFalse(staticIncludes({length: 3, get 0(){delete this[1]}, 1: 2}, 2));
assertFalse(staticIncludes({length: -100, 0: 1}, 1));

assert.throws(TypeError, () => staticIncludes());
assert.throws(TypeError, () => staticIncludes(null));
assert.throws(TypeError, () => staticIncludes({get length(){throw TypeError()}}));
assert.throws(TypeError, () => staticIncludes({length: 3, get 1() {throw TypeError()}}, 2));
assert.throws(TypeError, () => staticIncludes({__proto__: {get 1() {throw TypeError()}}, length: 3}, 2));
assert.throws(TypeError, () => staticIncludes(new Proxy([1], {get(){throw TypeError()}})));

function assertTrue(v) {
    assert.sameValue(v, true);
}

function assertFalse(v) {
    assert.sameValue(v, false);
}

function staticIncludes(o, v, f) {
    return [].includes.call(o, v, f);
}
