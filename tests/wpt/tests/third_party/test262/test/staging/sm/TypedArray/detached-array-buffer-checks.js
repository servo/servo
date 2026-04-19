// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/
// Nearly every %TypedArray%.prototype method should throw a TypeError when called
// atop a detached array buffer. Here we check verify that this holds true for
// all relevant functions.
let buffer = new ArrayBuffer(32);
let array  = new Int32Array(buffer);
$DETACHBUFFER(buffer);

// A nice poisoned callable to ensure that we fail on a detached buffer check
// before a method attempts to do anything with its arguments.
var POISON = (function() {
    var internalTarget = {};
    var throwForAllTraps =
    new Proxy(internalTarget, { get(target, prop, receiver) {
        assert.sameValue(target, internalTarget);
        assert.sameValue(receiver, throwForAllTraps);
        throw "FAIL: " + prop + " trap invoked";
    }});
    return new Proxy(throwForAllTraps, throwForAllTraps);
});


assert.throws(TypeError, () => {
    array.copyWithin(POISON);
});

assert.throws(TypeError, () => {
    array.entries();
});

assert.throws(TypeError, () => {
    array.fill(POISON);
});

assert.throws(TypeError, () => {
    array.filter(POISON);
});

assert.throws(TypeError, () => {
    array.find(POISON);
});

assert.throws(TypeError, () => {
    array.findIndex(POISON);
});

assert.throws(TypeError, () => {
    array.forEach(POISON);
});

assert.throws(TypeError, () => {
    array.indexOf(POISON);
});

assert.throws(TypeError, () => {
    array.includes(POISON);
});

assert.throws(TypeError, () => {
    array.join(POISON);
});

assert.throws(TypeError, () => {
    array.keys();
});

assert.throws(TypeError, () => {
    array.lastIndexOf(POISON);
});

assert.throws(TypeError, () => {
    array.map(POISON);
});

assert.throws(TypeError, () => {
    array.reduce(POISON);
});

assert.throws(TypeError, () => {
    array.reduceRight(POISON);
});

assert.throws(TypeError, () => {
    array.reverse();
});

assert.throws(TypeError, () => {
    array.slice(POISON, POISON);
});

assert.throws(TypeError, () => {
    array.some(POISON);
});

assert.throws(TypeError, () => {
    array.values();
});

assert.throws(TypeError, () => {
    array.every(POISON);
});

assert.throws(TypeError, () => {
    array.sort(POISON);
});

