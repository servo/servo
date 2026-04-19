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
// If the mapfn argument to Array.from is undefined, don't map.
assert.deepEqual(Array.from([3, 4, 5], undefined), [3, 4, 5]);
assert.deepEqual(Array.from([4, 5, 6], undefined, Math), [4, 5, 6]);

// mapfn is called with two arguments: value and index.
var log = [];
function f() {
    log.push(Array.from(arguments));
    return log.length;
}
assert.deepEqual(Array.from(['a', 'e', 'i', 'o', 'u'], f), [1, 2, 3, 4, 5]);
assert.deepEqual(log, [['a', 0], ['e', 1], ['i', 2], ['o', 3], ['u', 4]]);

// If the object to be copied is non-iterable, mapfn is still called with two
// arguments.
log = [];
assert.deepEqual(Array.from({0: "zero", 1: "one", length: 2}, f), [1, 2]);
assert.deepEqual(log, [["zero", 0], ["one", 1]]);

// If the object to be copied is iterable and the constructor is not Array,
// mapfn is still called with two arguments.
log = [];
function C() {}
C.from = Array.from;
var c = new C;
c[0] = 1;
c[1] = 2;
c.length = 2;
assert.deepEqual(C.from(["zero", "one"], f), c);
assert.deepEqual(log, [["zero", 0], ["one", 1]]);

// The mapfn is called even if the value to be mapped is undefined.
assert.deepEqual(Array.from([0, 1, , 3], String), ["0", "1", "undefined", "3"]);
var arraylike = {length: 4, "0": 0, "1": 1, "3": 3};
assert.deepEqual(Array.from(arraylike, String), ["0", "1", "undefined", "3"]);

