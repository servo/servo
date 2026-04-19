/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
features: [Symbol]
---*/

assert.sameValue("values" in Object, true);
assert.sameValue(Object.values.length, 1);

var o, values;

o = { a: 3, b: 2 };
values = Object.values(o);
assert.compareArray(values, [3, 2]);

o = { get a() { return 17; }, b: 2 };
values = Object.values(o),
assert.compareArray(values, [17, 2]);

o = { __iterator__: function() { throw new Error("non-standard __iterator__ called?"); } };
values = Object.values(o);
assert.compareArray(values, [o.__iterator__]);

o = { a: 1, b: 2 };
delete o.a;
o.a = 3;
values = Object.values(o);
assert.compareArray(values, [2, 3]);

o = [0, 1, 2];
values = Object.values(o);
assert.compareArray(values, [0, 1, 2]);

o = /./.exec("abc");
values = Object.values(o);
assert.compareArray(values, ["a", 0, "abc", undefined]);

o = { a: 1, b: 2, c: 3 };
delete o.b;
o.b = 5;
values = Object.values(o);
assert.compareArray(values, [1, 3, 5]);

function f() { }
f.prototype.p = 1;
o = new f();
o.g = 1;
values = Object.values(o);
assert.compareArray(values, [1]);

var o = {get a() {delete this.b; return 1}, b: 2, c: 3};
values = Object.values(o);
assert.compareArray(values, [1, 3]);

assert.throws(TypeError, () => Object.values());
assert.throws(TypeError, () => Object.values(undefined));
assert.throws(TypeError, () => Object.values(null));

assert.compareArray(Object.values(1), []);
assert.compareArray(Object.values(true), []);
assert.compareArray(Object.values(Symbol("foo")), []);

assert.compareArray(Object.values("foo"), ["f", "o", "o"]);

values = Object.values({
    get a(){
        Object.defineProperty(this, "b", {enumerable: false});
        return "A";
    },
    b: "B"
});
assert.compareArray(values, ["A"]);

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
values = Object.values(o);
assert.sameValue(ownKeysCallCount, 1);
assert.compareArray(values, [3, 1]);
assert.compareArray(getOwnPropertyDescriptorCalls, ["c", "a"]);
