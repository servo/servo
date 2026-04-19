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
// We should preserve holes when sorting sparce arrays.
// See bug: 1246860

function denseCount(arr) {
    var c = 0;
    for (var i = 0; i < arr.length; i++)
        if (i in arr)
            c++;
    return c;
}

let a = [,,,,,,,,,,,,,,,,,,,,{size: 1},{size: 2}];
let b = [,,,,,,,,,,,,,,,,,,,,{size: 1},{size: 2}].sort();
let c = [,,,,,,,,,,,,,,,,,,,,{size: 1},{size: 2}].sort((a, b) => {+a.size - +b.size});

assert.sameValue(a.length, 22);
assert.sameValue(denseCount(a), 2);
assert.sameValue(a.length, b.length);
assert.sameValue(b.length, c.length);
assert.sameValue(denseCount(a), denseCount(b));
assert.sameValue(denseCount(b), denseCount(c));

let superSparce = new Array(5000);
superSparce[0] = 99;
superSparce[4000] = 0;
superSparce[4999] = -1;

assert.sameValue(superSparce.length, 5000);
assert.sameValue(denseCount(superSparce), 3);

superSparce.sort((a, b) => 1*a-b);
assert.sameValue(superSparce.length, 5000);
assert.sameValue(denseCount(superSparce), 3);
assert.sameValue(superSparce[0], -1);
assert.sameValue(superSparce[1], 0);
assert.sameValue(superSparce[2], 99);

let allHoles = new Array(2600);
assert.sameValue(allHoles.length, 2600);
assert.sameValue(denseCount(allHoles), 0);
allHoles.sort((a, b) => 1*a-b);
assert.sameValue(allHoles.length, 2600);
assert.sameValue(denseCount(allHoles), 0);

let oneHole = new Array(2600);
oneHole[1399] = {size: 27};
assert.sameValue(oneHole.length, 2600);
assert.sameValue(denseCount(oneHole), 1);
oneHole.sort((a, b) => {+a.size - +b.size});
assert.deepEqual(oneHole[0], {size: 27});
assert.sameValue(oneHole.length, 2600);
assert.sameValue(denseCount(oneHole), 1);

// Sealed objects should be sortable, including those with holes (so long
// as the holes appear at the end, so that they don't need to be moved).
assert.deepEqual(Object.seal([0, 99, -1]).sort((x, y) => 1 * x - y),
             Object.seal([-1, 0, 99]));

assert.deepEqual(Object.seal([1, 5, 4, , ,]).sort((x, y) => 1 * x - y),
             Object.seal([1, 4, 5, , ,]));

