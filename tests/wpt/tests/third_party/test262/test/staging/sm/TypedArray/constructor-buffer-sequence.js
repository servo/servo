// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/
// 22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

// Ensure the various error conditions are tested in the correct order.

const otherGlobal = $262.createRealm().global;

function* createBuffers(lengths = [0, 8]) {
    for (let length of lengths) {
        let buffer = new ArrayBuffer(length);
        yield {buffer, detach: () => $DETACHBUFFER(buffer)};
    }

    for (let length of lengths) {
        let buffer = new otherGlobal.ArrayBuffer(length);
        yield {buffer, detach: () => otherGlobal.$262.detachArrayBuffer(buffer)};
    }
}

const poisonedValue = new Proxy({}, new Proxy({}, {
    get() {
        // Throws an exception when any proxy trap is invoked.
        throw new Error("Poisoned Value");
    }
}));

class ExpectedError extends Error { }

function ConstructorWithThrowingPrototype(detach) {
    return Object.defineProperty(function(){}.bind(null), "prototype", {
        get() {
            if (detach)
                detach();
            throw new ExpectedError();
        }
    });
}

function ValueThrowing(detach) {
    return {
        valueOf() {
            if (detach)
                detach();
            throw new ExpectedError();
        }
    };
}

function ValueReturning(value, detach) {
    return {
        valueOf() {
            if (detach)
                detach();
            return value;
        }
    };
}

// Ensure step 4 |AllocateTypedArray| is executed before step 6 |ToIndex(byteOffset)|.
for (let {buffer} of createBuffers()) {
    let constructor = ConstructorWithThrowingPrototype();

    assert.throws(ExpectedError, () =>
        Reflect.construct(Int32Array, [buffer, poisonedValue, 0], constructor));
}

// Ensure step 4 |AllocateTypedArray| is executed before step 9 |IsDetachedBuffer(buffer)|.
for (let {buffer, detach} of createBuffers()) {
    let constructor = ConstructorWithThrowingPrototype();

    detach();
    assert.throws(ExpectedError, () =>
        Reflect.construct(Int32Array, [buffer, 0, 0], constructor));
}

// Ensure step 4 |AllocateTypedArray| is executed before step 9 |IsDetachedBuffer(buffer)|.
// - Variant: Detach buffer dynamically.
for (let {buffer, detach} of createBuffers()) {
    let constructor = ConstructorWithThrowingPrototype(detach);

    assert.throws(ExpectedError, () =>
        Reflect.construct(Int32Array, [buffer, 0, 0], constructor));
}

// Ensure step 4 |AllocateTypedArray| is executed before step 8.a |ToIndex(length)|.
for (let {buffer} of createBuffers()) {
    let constructor = ConstructorWithThrowingPrototype();

    assert.throws(ExpectedError, () =>
        Reflect.construct(Int32Array, [buffer, 0, poisonedValue], constructor));
}

// Ensure step 6 |ToIndex(byteOffset)| is executed before step 9 |IsDetachedBuffer(buffer)|.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = ValueThrowing();

    detach();
    assert.throws(ExpectedError, () => new Int32Array(buffer, byteOffset, 0));
}

// Ensure step 6 |ToIndex(byteOffset)| is executed before step 9 |IsDetachedBuffer(buffer)|.
// - Variant: Detach buffer dynamically.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = ValueThrowing(detach);

    assert.throws(ExpectedError, () => new Int32Array(buffer, byteOffset, 0));
}

// Ensure step 6 |ToIndex(byteOffset)| is executed before step 8.a |ToIndex(length)|.
for (let {buffer} of createBuffers()) {
    let byteOffset = ValueThrowing();

    assert.throws(ExpectedError, () => new Int32Array(buffer, byteOffset, poisonedValue));
}

// Ensure step 7 |offset modulo elementSize ≠ 0| is executed before step 9 |IsDetachedBuffer(buffer)|.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = 1;

    detach();
    assert.throws(RangeError, () => new Int32Array(buffer, byteOffset, 0));
}

// Ensure step 7 |offset modulo elementSize ≠ 0| is executed before step 9 |IsDetachedBuffer(buffer)|.
// - Variant: Detach buffer dynamically.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = ValueReturning(1, detach);

    assert.throws(RangeError, () => new Int32Array(buffer, byteOffset, 0));
}

// Ensure step 7 |offset modulo elementSize ≠ 0| is executed before step 8.a |ToIndex(length)|.
for (let {buffer} of createBuffers()) {
    assert.throws(RangeError, () => new Int32Array(buffer, 1, poisonedValue));
}

// Ensure step 8.a |ToIndex(length)| is executed before step 9 |IsDetachedBuffer(buffer)|.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = 0;
    let length = ValueThrowing();

    detach();
    assert.throws(ExpectedError, () => new Int32Array(buffer, byteOffset, length));
}

// Ensure step 8.a |ToIndex(length)| is executed before step 9 |IsDetachedBuffer(buffer)|.
// - Variant: Detach buffer dynamically (1).
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = ValueReturning(0, detach);
    let length = ValueThrowing();

    assert.throws(ExpectedError, () => new Int32Array(buffer, byteOffset, length));
}

// Ensure step 8.a |ToIndex(length)| is executed before step 9 |IsDetachedBuffer(buffer)|.
// - Variant: Detach buffer dynamically (2).
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = 0;
    let length = ValueThrowing(detach);

    assert.throws(ExpectedError, () => new Int32Array(buffer, byteOffset, length));
}

// Ensure step 9 |IsDetachedBuffer(buffer)| is executed before step 11.a |bufferByteLength modulo elementSize ≠ 0|.
for (let {buffer, detach} of createBuffers([1, 9])) {
    let byteOffset = 0;

    detach();
    assert.throws(TypeError, () => new Int32Array(buffer, byteOffset));
}

// Ensure step 9 |IsDetachedBuffer(buffer)| is executed before step 11.a |bufferByteLength modulo elementSize ≠ 0|.
// - Variant: Detach buffer dynamically.
for (let {buffer, detach} of createBuffers([1, 9])) {
    let byteOffset = ValueReturning(0, detach);

    assert.throws(TypeError, () => new Int32Array(buffer, byteOffset));
}

// Ensure step 9 |IsDetachedBuffer(buffer)| is executed before step 11.c |newByteLength < 0|.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = 64;

    detach();
    assert.throws(TypeError, () => new Int32Array(buffer, byteOffset));
}

// Ensure step 9 |IsDetachedBuffer(buffer)| is executed before step 11.c |newByteLength < 0|.
// - Variant: Detach buffer dynamically.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = ValueReturning(64, detach);

    assert.throws(TypeError, () => new Int32Array(buffer, byteOffset));
}

// Ensure step 9 |IsDetachedBuffer(buffer)| is executed before step 12.b |offset+newByteLength > bufferByteLength|.
// - Case A: The given byteOffset is too large.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = 64;
    let length = ValueReturning(0, detach);

    assert.throws(TypeError, () => new Int32Array(buffer, byteOffset, length));
}

// Ensure step 9 |IsDetachedBuffer(buffer)| is executed before step 12.b |offset+newByteLength > bufferByteLength|.
// - Case B: The given length is too large.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = 0;
    let length = ValueReturning(64, detach);

    assert.throws(TypeError, () => new Int32Array(buffer, byteOffset, length));
}

// Ensure we handle the case when ToIndex(byteOffset) detaches the array buffer.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = ValueReturning(0, detach);
    let length = 0;

    assert.throws(TypeError, () => new Int32Array(buffer, byteOffset, length));
}

// Ensure we handle the case when ToIndex(length) detaches the array buffer.
for (let {buffer, detach} of createBuffers()) {
    let byteOffset = 0;
    let length = ValueReturning(0, detach);

    assert.throws(TypeError, () => new Int32Array(buffer, byteOffset, length));
}

