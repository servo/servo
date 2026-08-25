// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/

// Tests for detached ArrayBuffer checks in %TypedArray%.prototype.set(array|typedArray, offset).

function* createTypedArrays(lengths = [0, 1, 4, 4096]) {
    for (let length of lengths) {
        let buffer = new ArrayBuffer(length * Int32Array.BYTES_PER_ELEMENT);
        let typedArray = new Int32Array(buffer);

        yield {typedArray, buffer};
    }
}

class ExpectedError extends Error {}

// No detached check on function entry.
for (let {typedArray, buffer} of createTypedArrays()) {
    $DETACHBUFFER(buffer);

    assert.throws(ExpectedError, () => typedArray.set(null, {
        valueOf() {
            throw new ExpectedError();
        }
    }));
}

// Check for detached buffer after calling ToInteger(offset). Test with:
// - valid offset,
// - too large offset,
// - and negative offset.
for (let [offset, error] of [[0, TypeError], [1000000, TypeError], [-1, RangeError]]) {
    for (let source of [[], [0], new Int32Array(0), new Int32Array(1)]) {
        for (let {typedArray, buffer} of createTypedArrays()) {
            assert.throws(error, () => typedArray.set(source, {
                valueOf() {
                    $DETACHBUFFER(buffer);
                    return offset;
                }
            }));
        }
    }
}

// Tests when called with detached typed array as source.
for (let {typedArray} of createTypedArrays()) {
    for (let {typedArray: source, buffer: sourceBuffer} of createTypedArrays()) {
        $DETACHBUFFER(sourceBuffer);

        assert.throws(ExpectedError, () => typedArray.set(source, {
            valueOf() {
                throw new ExpectedError();
            }
        }));
    }
}

// Check when detaching source buffer in ToInteger(offset). Test with:
// - valid offset,
// - too large offset,
// - and negative offset.
for (let [offset, error] of [[0, TypeError], [1000000, TypeError], [-1, RangeError]]) {
    for (let {typedArray} of createTypedArrays()) {
        for (let {typedArray: source, buffer: sourceBuffer} of createTypedArrays()) {
            assert.throws(error, () => typedArray.set(source, {
                valueOf() {
                    $DETACHBUFFER(sourceBuffer);
                    return offset;
                }
            }));
        }
    }
}

// Test when target and source use the same underlying buffer and
// ToInteger(offset) detaches the buffer. Test with:
// - same typed array,
// - different typed array, but same element type,
// - and different element type.
for (let src of [ta => ta, ta => new Int32Array(ta.buffer), ta => new Float32Array(ta.buffer)]) {
    for (let {typedArray, buffer} of createTypedArrays()) {
        let source = src(typedArray);
        assert.throws(TypeError, () => typedArray.set(source, {
            valueOf() {
                $DETACHBUFFER(buffer);
                return 0;
            }
        }));
    }
}

// Test when Get(src, "length") detaches the buffer, but srcLength is 0.
// Also use different offsets to ensure bounds checks use the typed array's
// length value from before detaching the buffer.
for (let offset of [() => 0, ta => Math.min(1, ta.length), ta => Math.max(0, ta.length - 1)]) {
    for (let {typedArray, buffer} of createTypedArrays()) {
        let source = {
            get length() {
                $DETACHBUFFER(buffer);
                return 0;
            }
        };
        typedArray.set(source, offset(typedArray));
    }
}

// Test when ToLength(Get(src, "length")) detaches the buffer, but
// srcLength is 0. Also use different offsets to ensure bounds checks use
// the typed array's length value from before detaching the buffer.
for (let offset of [() => 0, ta => Math.min(1, ta.length), ta => Math.max(0, ta.length - 1)]) {
    for (let {typedArray, buffer} of createTypedArrays()) {
        let source = {
            length: {
                valueOf() {
                    $DETACHBUFFER(buffer);
                    return 0;
                }
            }
        };
        typedArray.set(source, offset(typedArray));
    }
}

// Test no TypeError is thrown when the typed array is detached and
// srcLength > 0.
for (let {typedArray, buffer} of createTypedArrays()) {
    let source = {
        length: {
            valueOf() {
                $DETACHBUFFER(buffer);
                return 1;
            }
        }
    };
    if (typedArray.length === 0) {
        assert.throws(RangeError, () => typedArray.set(source));
    } else {
        typedArray.set(source);
    }
}

// Same as above, but with side-effect when executing Get(src, "0").
for (let {typedArray, buffer} of createTypedArrays()) {
    let source = {
        get 0() {
            throw new ExpectedError();
        },
        length: {
            valueOf() {
                $DETACHBUFFER(buffer);
                return 1;
            }
        }
    };
    let err = typedArray.length === 0 ? RangeError : ExpectedError;
    assert.throws(err, () => typedArray.set(source));
}

// Same as above, but with side-effect when executing ToNumber(Get(src, "0")).
for (let {typedArray, buffer} of createTypedArrays()) {
    let source = {
        get 0() {
            return {
                valueOf() {
                    throw new ExpectedError();
                }
            };
        },
        length: {
            valueOf() {
                $DETACHBUFFER(buffer);
                return 1;
            }
        }
    };
    let err = typedArray.length === 0 ? RangeError : ExpectedError;
    assert.throws(err, () => typedArray.set(source));
}

// Side-effects when getting the source elements detach the buffer.
for (let {typedArray, buffer} of createTypedArrays()) {
    let source = Object.defineProperties([], {
        0: {
            get() {
                $DETACHBUFFER(buffer);
                return 1;
            }
        }
    });
    if (typedArray.length === 0) {
        assert.throws(RangeError, () => typedArray.set(source));
    } else {
        typedArray.set(source);
    }
}

// Side-effects when getting the source elements detach the buffer. Also
// ensure other elements are accessed.
for (let {typedArray, buffer} of createTypedArrays()) {
    let accessed = false;
    let source = Object.defineProperties([], {
        0: {
            get() {
                $DETACHBUFFER(buffer);
                return 1;
            }
        },
        1: {
            get() {
                assert.sameValue(accessed, false);
                accessed = true;
                return 2;
            }
        }
    });
    if (typedArray.length <= 1) {
        assert.throws(RangeError, () => typedArray.set(source));
    } else {
        assert.sameValue(accessed, false);
        typedArray.set(source);
        assert.sameValue(accessed, true);
    }
}

// Side-effects when converting the source elements detach the buffer.
for (let {typedArray, buffer} of createTypedArrays()) {
    let source = [{
        valueOf() {
            $DETACHBUFFER(buffer);
            return 1;
        }
    }];
    if (typedArray.length === 0) {
        assert.throws(RangeError, () => typedArray.set(source));
    } else {
        typedArray.set(source);
    }
}

// Side-effects when converting the source elements detach the buffer. Also
// ensure other elements are accessed.
for (let {typedArray, buffer} of createTypedArrays()) {
    let accessed = false;
    let source = [{
        valueOf() {
            $DETACHBUFFER(buffer);
            return 1;
        }
    }, {
        valueOf() {
            assert.sameValue(accessed, false);
            accessed = true;
            return 2;
        }
    }];
    if (typedArray.length <= 1) {
        assert.throws(RangeError, () => typedArray.set(source));
    } else {
        assert.sameValue(accessed, false);
        typedArray.set(source);
        assert.sameValue(accessed, true);
    }
}
