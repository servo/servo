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
// Array.from can be applied to any constructor.
// For example, the Date builtin constructor.
var d = Array.from.call(Date, ["A", "B"]);
assert.sameValue(Array.isArray(d), false);
assert.sameValue(Object.prototype.toString.call(d), "[object Date]");
assert.sameValue(Object.getPrototypeOf(d), Date.prototype);
assert.sameValue(d.length, 2);
assert.sameValue(d[0], "A");
assert.sameValue(d[1], "B");

// Or Object.
var obj = Array.from.call(Object, []);
assert.sameValue(Array.isArray(obj), false);
assert.sameValue(Object.getPrototypeOf(obj), Object.prototype);
assert.sameValue(Object.getOwnPropertyNames(obj).join(","), "length");
assert.sameValue(obj.length, 0);

// Or any JS function.
function C(arg) {
    this.args = arguments;
}
var c = Array.from.call(C, {length: 1, 0: "zero"});
assert.sameValue(c instanceof C, true);
assert.sameValue(c.args.length, 1);
assert.sameValue(c.args[0], 1);
assert.sameValue(c.length, 1);
assert.sameValue(c[0], "zero");

// If the 'this' value passed to Array.from is not a constructor,
// a plain Array is created.
var arr = [3, 4, 5];
var nonconstructors = [
    {}, Math, Object.getPrototypeOf, undefined, 17,
    () => ({})  // arrow functions are not constructors
];
for (var v of nonconstructors) {
    obj = Array.from.call(v, arr);
    assert.sameValue(Array.isArray(obj), true);
    assert.deepEqual(obj, arr);
}

// Array.from does not get confused if global.Array is replaced with another
// constructor.
function NotArray() {
}
var RealArray = Array;
NotArray.from = Array.from;
Array = NotArray;
assert.sameValue(RealArray.from([1]) instanceof RealArray, true);
assert.sameValue(NotArray.from([1]) instanceof NotArray, true);

