// META: title=test resample2d output shape agrees with the backend across a gather
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-resample2d-method
// https://www.w3.org/TR/webnn/#api-mlgraphbuilder-gather-method
//
// Regression test for a resample2d shape divergence. The output size is
// floor(input size * scale), which WebNN validates in double precision. A
// backend that re-derives it in float32 disagrees above 2^24, where float32
// cannot represent consecutive integers: for size 2^24 + 1 and scale 1.0 it
// rounds down to 2^24, under-allocating the axis by one element. A downstream
// gather, clamped to WebNN's (larger) shape, then reads out of bounds at the
// last index.
//
// The assertion does not check for a specific value, since backends may sample
// different values near 2^24. Instead it exposes `resampled` as a second output
// and checks that gathering the last index reads the same element as a direct
// read of it. That holds only when the gather stayed in bounds, i.e. when the
// shapes agree; an under-allocated axis clamps the gather elsewhere and breaks
// the equality.

// 2^24 + 1: the smallest positive integer float32 cannot represent exactly.
// Input byte length is 4 * 16777217 ~= 64 MiB, below the tensor byte length
// limit.
const kResampledDim = 16777217;
const kLastIndex = kResampledDim - 1;  // Largest index WebNN admits: 16777216.

let mlContext;

promise_setup(async () => {
  assert_implements(navigator.ml, 'missing navigator.ml');
  mlContext = await navigator.ml.createContext(contextOptions);
});

promise_test(async () => {
  const builder = new MLGraphBuilder(mlContext);

  // Rank-4 input with the large dimension on axis 2.
  const input = builder.input(
      'input', {dataType: 'float32', shape: [1, 1, kResampledDim, 1]});

  // Identity resample (scale 1.0). WebNN's validated output shape keeps axis 2
  // at kResampledDim; a float32 re-derivation would shrink it by one.
  const resampled = builder.resample2d(
      input, {mode: 'nearest-neighbor', scales: [1, 1], axes: [2, 3]});
  assert_equals(resampled.shape[2], kResampledDim,
                'resample2d output preserves the axis-2 dimension');

  // Gather the last WebNN-valid element along the resampled axis.
  const indices = builder.input('indices', {dataType: 'int32', shape: [1]});
  const gathered = builder.gather(resampled, indices, {axis: 2});
  assert_array_equals(gathered.shape, [1, 1, 1, 1], 'gather output shape');

  const [inputTensor, indicesTensor, resampledTensor, gatheredTensor, mlGraph] =
      await Promise.all([
        mlContext.createTensor({
          dataType: 'float32',
          shape: [1, 1, kResampledDim, 1],
          writable: true,
        }),
        mlContext.createTensor({dataType: 'int32', shape: [1], writable: true}),
        mlContext.createTensor({
          dataType: 'float32',
          shape: [1, 1, kResampledDim, 1],
          readable: true,
        }),
        mlContext.createTensor(
            {dataType: 'float32', shape: [1, 1, 1, 1], readable: true}),
        // Expose `resampled` as an output so its last element can be read back
        // and compared against the gather result.
        builder.build({'resampled': resampled, 'gathered': gathered}),
      ]);

  // Distinct non-zero sentinels at kLastIndex and kLastIndex - 1. On a
  // correctly sized backend the gather reads kLastIndex in bounds, so
  // gathered[0] equals a direct read of resampled[kLastIndex] regardless of the
  // values. If the axis is under-allocated, both the gather and that direct
  // read touch an element the backend never produced; the resulting behavior is
  // backend-defined (it may throw, clamp, or read adjacent memory). The
  // distinct sentinels give a mismatch something to show - e.g. a backend that
  // clamps the gather to its last valid index would read a different sentinel
  // than the unwritten kLastIndex. This is best-effort: a zero-filled input
  // could let both sides read 0 and hide the regression. The exact values are
  // never asserted; they only need to differ.
  const inputData = new Float32Array(kResampledDim);
  inputData[kLastIndex - 1] = 5;
  inputData[kLastIndex] = 42;
  mlContext.writeTensor(inputTensor, inputData);
  mlContext.writeTensor(indicesTensor, new Int32Array([kLastIndex]));

  mlContext.dispatch(
      mlGraph, {'input': inputTensor, 'indices': indicesTensor},
      {'resampled': resampledTensor, 'gathered': gatheredTensor});

  const resampledData =
      new Float32Array(await mlContext.readTensor(resampledTensor));
  const gatheredData =
      new Float32Array(await mlContext.readTensor(gatheredTensor));

  assert_equals(
      gatheredData[0], resampledData[kLastIndex],
      'gather at the last WebNN-valid index reads the same element as a direct ' +
          'read of that index, proving the resample2d output shape matches the ' +
          'backend allocation (no out-of-bounds clamp)');
}, 'resample2d output shape (from scales) matches backend allocation at the 2^24 boundary');
