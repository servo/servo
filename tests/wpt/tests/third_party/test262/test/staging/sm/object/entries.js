/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
features: [Symbol]
---*/

assert.sameValue("entries" in Object, true);
assert.sameValue(Object.entries.length, 1);

var o, entries;

o = { a: 3, b: 2 };
entries = Object.entries(o);
assert.deepEqual(entries, [["a", 3], ["b", 2]]);

o = { get a() { return 17; }, b: 2 };
entries = Object.entries(o),
assert.deepEqual(entries, [["a", 17], ["b", 2]]);

o = { __iterator__: function() { throw new Error("non-standard __iterator__ called?"); } };
entries = Object.entries(o);
assert.deepEqual(entries, [["__iterator__", o.__iterator__]]);

o = { a: 1, b: 2 };
delete o.a;
o.a = 3;
entries = Object.entries(o);
assert.deepEqual(entries, [["b", 2], ["a", 3]]);

o = [0, 1, 2];
entries = Object.entries(o);
assert.deepEqual(entries, [["0", 0], ["1", 1], ["2", 2]]);

o = /./.exec("abc");
entries = Object.entries(o);
assert.deepEqual(entries, [["0", "a"], ["index", 0], ["input", "abc"], ["groups", undefined]]);

o = { a: 1, b: 2, c: 3 };
delete o.b;
o.b = 5;
entries = Object.entries(o);
assert.deepEqual(entries, [["a", 1], ["c", 3], ["b", 5]]);

function f() { }
f.prototype.p = 1;
o = new f();
o.g = 1;
entries = Object.entries(o);
assert.deepEqual(entries, [["g", 1]]);

var o = {get a() {delete this.b; return 1}, b: 2, c: 3};
entries = Object.entries(o);
assert.deepEqual(entries, [["a", 1], ["c", 3]]);

assert.throws(TypeError, () => Object.entries());
assert.throws(TypeError, () => Object.entries(undefined));
assert.throws(TypeError, () => Object.entries(null));

assert.deepEqual(Object.entries(1), []);
assert.deepEqual(Object.entries(true), []);
assert.deepEqual(Object.entries(Symbol("foo")), []);

assert.deepEqual(Object.entries("foo"), [["0", "f"], ["1", "o"], ["2", "o"]]);

entries = Object.entries({
    get a(){
        Object.defineProperty(this, "b", {enumerable: false});
        return "A";
    },
    b: "B"
});
assert.deepEqual(entries, [["a", "A"]]);

let ownKeysCallCount = 0;
let getOwnPropertyDescriptorCalls = [];
let target = { a: 1, b: 2, c: 3 };
o = new Proxy(target, {
    ownKeys() {
        ownKeysCallCount++;
        return ["c", "a"];
    },
    getOwnPropertyDescriptor(target, key) {
        getOwnPropertyDescriptorCalls.push(key);
        return Object.getOwnPropertyDescriptor(target, key);
    }
});
entries = Object.entries(o);
assert.sameValue(ownKeysCallCount, 1);
assert.deepEqual(entries, [["c", 3], ["a", 1]]);
assert.deepEqual(getOwnPropertyDescriptorCalls, ["c", "a"]);
