// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Set values from different instances using the different buffer and different
  type.
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, SharedArrayBuffer, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sab = new SharedArrayBuffer(2 * BigInt64Array.BYTES_PER_ELEMENT);
  var otherCtor = TA === BigInt64Array ? BigUint64Array : BigInt64Array;
  var src = new otherCtor(sab);
  src[0] = 42n;
  src[1] = 43n;
  var sample, result;

  sample = new TA(makeCtorArg([1n, 2n, 3n, 4n]));
  result = sample.set(src, 0);
  assert(compareArray(sample, [42n, 43n, 3n, 4n]), "src is SAB-backed, offset: 0, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1n, 2n, 3n, 4n]));
  result = sample.set(src, 1);
  assert(compareArray(sample, [1n, 42n, 43n, 4n]), "src is SAB-backed, offset: 1, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1n, 2n, 3n, 4n]));
  result = sample.set(src, 2);
  assert(compareArray(sample, [1n, 2n, 42n, 43n]), "src is SAB-backed, offset: 2, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  src = new BigInt64Array([42n, 43n]);

  sab = new SharedArrayBuffer(4 * TA.BYTES_PER_ELEMENT);
  sample = new TA(sab);
  sample[0] = 1n;
  sample[1] = 2n;
  sample[2] = 3n;
  sample[3] = 4n;
  result = sample.set(src, 0);
  assert(compareArray(sample, [42n, 43n, 3n, 4n]), "sample is SAB-backed, offset: 0, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sab = new SharedArrayBuffer(4 * TA.BYTES_PER_ELEMENT);
  sample = new TA(sab);
  sample[0] = 1n;
  sample[1] = 2n;
  sample[2] = 3n;
  sample[3] = 4n;
  result = sample.set(src, 1);
  assert(compareArray(sample, [1n, 42n, 43n, 4n]), "sample is SAB-backed, offset: 1, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sab = new SharedArrayBuffer(4 * TA.BYTES_PER_ELEMENT);
  sample = new TA(sab);
  sample[0] = 1n;
  sample[1] = 2n;
  sample[2] = 3n;
  sample[3] = 4n;
  result = sample.set(src, 2);
  assert(compareArray(sample, [1n, 2n, 42n, 43n]), "sample is SAB-backed, offset: 2, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  var sab1 = new SharedArrayBuffer(2 * BigInt64Array.BYTES_PER_ELEMENT);
  src = new BigInt64Array(sab1);
  src[0] = 42n;
  src[1] = 43n;

  var sab2;
  sab2 = new SharedArrayBuffer(4 * TA.BYTES_PER_ELEMENT);
  sample = new TA(sab2);
  sample[0] = 1n;
  sample[1] = 2n;
  sample[2] = 3n;
  sample[3] = 4n;
  result = sample.set(src, 0);
  assert(compareArray(sample, [42n, 43n, 3n, 4n]), "src and sample are SAB-backed, offset: 0, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sab2 = new SharedArrayBuffer(4 * TA.BYTES_PER_ELEMENT);
  sample = new TA(sab2);
  sample[0] = 1n;
  sample[1] = 2n;
  sample[2] = 3n;
  sample[3] = 4n;
  result = sample.set(src, 1);
  assert(compareArray(sample, [1n, 42n, 43n, 4n]), "src and sample are SAB-backed, offset: 1, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sab2 = new SharedArrayBuffer(4 * TA.BYTES_PER_ELEMENT);
  sample = new TA(sab2);
  sample[0] = 1n;
  sample[1] = 2n;
  sample[2] = 3n;
  sample[3] = 4n;
  result = sample.set(src, 2);
  assert(compareArray(sample, [1n, 2n, 42n, 43n]), "src and sample are SAB-backed, offset: 2, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");
});
