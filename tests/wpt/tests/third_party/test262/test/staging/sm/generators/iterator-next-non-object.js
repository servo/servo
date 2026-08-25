// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  IteratorNext should throw if the value returned by iterator.next() is not an object.
info: bugzilla.mozilla.org/show_bug.cgi?id=1016936
esid: pending
---*/

var nonobjs = [
    null,
    undefined,
    1,
    true,
    "a",
    Symbol.iterator,
];

function createIterable(v) {
    var iterable = {};
    iterable[Symbol.iterator] = function () {
        return {
            next: function () {
                return v;
            }
        };
    };
    return iterable;
}

function f() {
}

for (var nonobj of nonobjs) {
    var iterable = createIterable(nonobj);

    assert.throws(TypeError, () => [...iterable]);
    assert.throws(TypeError, () => f(...iterable));

    assert.throws(TypeError, () => { for (var x of iterable) {} });

    assert.throws(TypeError, () => {
        var [a] = iterable;
    });
    assert.throws(TypeError, () => {
        var [...a] = iterable;
    });

    assert.throws(TypeError, () => Array.from(iterable));
    assert.throws(TypeError, () => new Map(iterable));
    assert.throws(TypeError, () => new Set(iterable));
    assert.throws(TypeError, () => new WeakMap(iterable));
    assert.throws(TypeError, () => new WeakSet(iterable));
    assert.throws(TypeError, () => new Int8Array(iterable));
    assert.throws(TypeError, () => Int8Array.from(iterable));

    assert.throws(TypeError, () => {
        var g = function*() {
            yield* iterable;
        };
        var v = g();
        v.next();
    });
}
