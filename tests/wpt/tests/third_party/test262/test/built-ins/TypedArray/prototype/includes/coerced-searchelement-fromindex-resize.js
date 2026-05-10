// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.includes
description: >
  TypedArray.p.includes behaves correctly on TypedArrays backed by resizable
  buffers that are resized during argument coercion.
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer, Array.prototype.includes]
---*/

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert(!fixedLength.includes(undefined));
  // The TA is OOB so it includes only "undefined".
  assert(fixedLength.includes(undefined, evil));
}
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  let n0 = MayNeedBigInt(fixedLength, 0);
  assert(fixedLength.includes(n0));
  // The TA is OOB so it includes only "undefined".
  assert(!fixedLength.includes(n0, evil));
}
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  let evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert(!lengthTracking.includes(undefined));
  // "includes" iterates until the original length and sees "undefined"s.
  assert(lengthTracking.includes(undefined, evil));
}
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    lengthTracking[i] = MayNeedBigInt(lengthTracking, 1);
  }
  let evil = {
    valueOf: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  let n0 = MayNeedBigInt(lengthTracking, 0);
  assert(!lengthTracking.includes(n0));
  // The TA grew but we only look at the data until the original length.
  assert(!lengthTracking.includes(n0, evil));
}
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  lengthTracking[0] = MayNeedBigInt(lengthTracking, 1);
  let evil = {
    valueOf: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return -4;
    }
  };
  let n1 = MayNeedBigInt(lengthTracking, 1);
  assert(lengthTracking.includes(n1, -4));
  // The TA grew but the start index conversion is done based on the original
  // length.
  assert(lengthTracking.includes(n1, evil));
}
