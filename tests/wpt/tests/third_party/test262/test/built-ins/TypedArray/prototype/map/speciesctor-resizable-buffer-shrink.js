// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.map
description: >
  TypedArray.p.map behaves correctly on TypedArrays backed by resizable buffers
  that are shrunk by the species constructor.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Returns a function that collects an appropriate typed array into values. Such
// a result can be used as an argument to .map.
function CollectWithUndefined(values) {
  return (n, ix, ta) => {
    if (typeof n == 'bigint') {
      values.push(Number(n));
    } else {
      values.push(n);
    }
    if (ta instanceof BigInt64Array || ta instanceof BigUint64Array) {
      // We still need to return a valid BigInt / non-BigInt, even if
      // n is `undefined`.
      return 0n;
    }
    return 0;
  }
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  let resizeWhenConstructorCalled = false;
  class MyArray extends ctor {
    constructor(...params) {
      super(...params);
      if (resizeWhenConstructorCalled) {
        rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      }
    }
  }
  ;
  const fixedLength = new MyArray(rab, 0, 4);
  resizeWhenConstructorCalled = true;
  const values = [];
  fixedLength.map(CollectWithUndefined(values));
  assert.compareArray(values, [
    undefined,
    undefined,
    undefined,
    undefined
  ]);
  assert.sameValue(rab.byteLength, 2 * ctor.BYTES_PER_ELEMENT);
}
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const taWrite = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, i);
  }
  let resizeWhenConstructorCalled = false;
  class MyArray extends ctor {
    constructor(...params) {
      super(...params);
      if (resizeWhenConstructorCalled) {
        rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      }
    }
  }
  ;
  const lengthTracking = new MyArray(rab);
  resizeWhenConstructorCalled = true;
  const values = [];
  lengthTracking.map(CollectWithUndefined(values));
  assert.compareArray(values, [
    0,
    1,
    undefined,
    undefined
  ]);
  assert.sameValue(rab.byteLength, 2 * ctor.BYTES_PER_ELEMENT);
}
