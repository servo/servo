// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Throws a TypeError if typedArray arg is not an Int32Array
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  1. Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).

  ValidateSharedIntegerTypedArray ( typedArray [ , waitable ] )

  5. If waitable is true, then
    a. If typeName is not "Int32Array" or "BigInt64Array", throw a TypeError exception.

features: [Atomics.waitAsync, Float32Array, Float64Array, Int8Array, TypedArray, Uint16Array, Uint8Array, Uint8ClampedArray, arrow-function, SharedArrayBuffer, Atomics]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const poisoned = {
  valueOf() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(TypeError, () => {
  const view = new Float64Array(
    new SharedArrayBuffer(Float64Array.BYTES_PER_ELEMENT * 8)
  );
  Atomics.waitAsync(view, poisoned, poisoned, poisoned);
}, '`const view = new Float64Array( new SharedArrayBuffer(Float64Array.BYTES_PER_ELEMENT * 8) ); Atomics.waitAsync(view, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  const view = new Float32Array(
    new SharedArrayBuffer(Float32Array.BYTES_PER_ELEMENT * 4)
  );
  Atomics.waitAsync(view, poisoned, poisoned, poisoned);
}, '`const view = new Float32Array( new SharedArrayBuffer(Float32Array.BYTES_PER_ELEMENT * 4) ); Atomics.waitAsync(view, poisoned, poisoned, poisoned)` throws a TypeError exception');

if (typeof Float16Array !== 'undefined') {
  assert.throws(TypeError, function() {
    const view = new Float16Array(
      new SharedArrayBuffer(Float16Array.BYTES_PER_ELEMENT * 2)
    );
    Atomics.waitAsync(view, poisoned, poisoned, poisoned);
  }, '`const view = new Float16Array( new SharedArrayBuffer(Float16Array.BYTES_PER_ELEMENT * 2) ); Atomics.waitAsync(view, poisoned, poisoned, poisoned)` throws a TypeError exception');
}

assert.throws(TypeError, () => {
  const view = new Int16Array(
    new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 2)
  );
  Atomics.waitAsync(view, poisoned, poisoned, poisoned);
}, '`const view = new Int16Array( new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 2) ); Atomics.waitAsync(view, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  const view = new Int8Array(
    new SharedArrayBuffer(Int8Array.BYTES_PER_ELEMENT)
  );
  Atomics.waitAsync(view, poisoned, poisoned, poisoned);
}, '`const view = new Int8Array( new SharedArrayBuffer(Int8Array.BYTES_PER_ELEMENT) ); Atomics.waitAsync(view, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  const view = new Uint32Array(
    new SharedArrayBuffer(Uint32Array.BYTES_PER_ELEMENT * 4)
  );
  Atomics.waitAsync(view, poisoned, poisoned, poisoned);
}, '`const view = new Uint32Array( new SharedArrayBuffer(Uint32Array.BYTES_PER_ELEMENT * 4) ); Atomics.waitAsync(view, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  const view = new Uint16Array(
    new SharedArrayBuffer(Uint16Array.BYTES_PER_ELEMENT * 2)
  );
  Atomics.waitAsync(view, poisoned, poisoned, poisoned);
}, '`const view = new Uint16Array( new SharedArrayBuffer(Uint16Array.BYTES_PER_ELEMENT * 2) ); Atomics.waitAsync(view, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  const view = new Uint8Array(
    new SharedArrayBuffer(Uint8Array.BYTES_PER_ELEMENT)
  );
  Atomics.waitAsync(view, poisoned, poisoned, poisoned);
}, '`const view = new Uint8Array( new SharedArrayBuffer(Uint8Array.BYTES_PER_ELEMENT) ); Atomics.waitAsync(view, poisoned, poisoned, poisoned)` throws a TypeError exception');

assert.throws(TypeError, () => {
  const view = new Uint8ClampedArray(
    new SharedArrayBuffer(Uint8ClampedArray.BYTES_PER_ELEMENT)
  );
  Atomics.waitAsync(view, poisoned, poisoned, poisoned);
}, '`const view = new Uint8ClampedArray( new SharedArrayBuffer(Uint8ClampedArray.BYTES_PER_ELEMENT) ); Atomics.waitAsync(view, poisoned, poisoned, poisoned)` throws a TypeError exception');
