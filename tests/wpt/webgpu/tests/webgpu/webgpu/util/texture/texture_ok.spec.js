/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = 'checkPixels helpers behave as expected against real textures';import { makeTestGroup } from '../../../common/framework/test_group.js';
import { GPUTest } from '../../gpu_test.js';

import { TexelView } from './texel_view.js';
import { textureContentIsOKByT2B } from './texture_ok.js';

export const g = makeTestGroup(GPUTest);

g.test('float32').
desc(`Basic test that actual/expected must match, for float32.`).
params((u) =>
u.
combineWithParams([
{ format: 'rgba32float' }, //
{ format: 'rg32float' }]
).
beginSubcases().
combineWithParams([
// Expected data is 0.6 in all channels
{ data: 0.6, opts: { maxFractionalDiff: 0.0000001 }, _ok: true },
{ data: 0.6, opts: { maxDiffULPsForFloatFormat: 1 }, _ok: true },

{ data: 0.5999, opts: { maxFractionalDiff: 0 }, _ok: false },
{ data: 0.5999, opts: { maxFractionalDiff: 0.0001001 }, _ok: true },

{ data: 0.6001, opts: { maxFractionalDiff: 0 }, _ok: false },
{ data: 0.6001, opts: { maxFractionalDiff: 0.0001001 }, _ok: true },

{ data: 0.5999, opts: { maxDiffULPsForFloatFormat: 1677 }, _ok: false },
{ data: 0.5999, opts: { maxDiffULPsForFloatFormat: 1678 }, _ok: true },

{ data: 0.6001, opts: { maxDiffULPsForFloatFormat: 1676 }, _ok: false },
{ data: 0.6001, opts: { maxDiffULPsForFloatFormat: 1677 }, _ok: true }]
)
).
fn(async (t) => {
  const { format, data, opts, _ok } = t.params;

  const size = [1, 1];
  const texture = t.device.createTexture({
    format,
    size,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC
  });
  t.trackForCleanup(texture);
  t.device.queue.writeTexture({ texture }, new Float32Array([data, data, data, data]), {}, size);

  const expColor = { R: 0.6, G: 0.6, B: 0.6, A: 0.6 };
  const expTexelView = TexelView.fromTexelsAsColors(format, (_coords) => expColor);

  const result = await textureContentIsOKByT2B(t, { texture }, size, { expTexelView }, opts);
  t.expect(result === undefined === _ok, `expected ${_ok}, got ${result === undefined}`);
});

g.test('norm').
desc(`Basic test that actual/expected must match, for unorm/snorm.`).
params((u) =>
u.
combine('mode', ['bytes', 'colors']).
combineWithParams([
{ format: 'r8unorm', _maxValue: 255 },
{ format: 'r8snorm', _maxValue: 127 }]
).
beginSubcases().
combineWithParams([
// Expected data is [10, 10]
{ data: [10, 10], _ok: true },
{ data: [10, 11], _ok: false },
{ data: [11, 10], _ok: false },
{ data: [11, 11], _ok: false }]
)
).
fn(async (t) => {
  const { mode, format, _maxValue, data, _ok } = t.params;

  const size = [2, 1];
  const texture = t.device.createTexture({
    format,
    size,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC
  });
  t.trackForCleanup(texture);
  t.device.queue.writeTexture({ texture }, new Int8Array(data), {}, size);

  let expTexelView;
  switch (mode) {
    case 'bytes':
      expTexelView = TexelView.fromTexelsAsBytes(format, (_coords) => new Uint8Array([10]));
      break;
    case 'colors':
      expTexelView = TexelView.fromTexelsAsColors(format, (_coords) => ({ R: 10 / _maxValue }));
      break;
  }

  const result = await textureContentIsOKByT2B(
    t,
    { texture },
    size,
    { expTexelView },
    { maxDiffULPsForNormFormat: 0 }
  );
  t.expect(result === undefined === _ok, result?.message);
});

g.test('snorm_min').
desc(
  `The minimum snorm value has two possible representations (e.g. -127 and -128). Ensure that
    actual/expected can mismatch in both directions and pass the test.`
).
params((u) =>
u //
.combine('mode', ['bytes', 'colors']).
combineWithParams([
//
{ format: 'r8snorm', _maxValue: 127 }]
)
).
fn(async (t) => {
  const { mode, format, _maxValue } = t.params;

  const data = [-_maxValue, -_maxValue - 1];

  const size = [2, 1];
  const texture = t.device.createTexture({
    format,
    size,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC
  });
  t.trackForCleanup(texture);
  t.device.queue.writeTexture({ texture }, new Int8Array(data), {}, size);

  let expTexelView;
  switch (mode) {
    case 'bytes':
      {
        // Actual value should be [-127,-128], expected value is [-128,-127], both should pass.
        const exp = [-_maxValue - 1, -_maxValue];
        expTexelView = TexelView.fromTexelsAsBytes(
          format,
          (coords) => new Uint8Array([exp[coords.x]])
        );
      }
      break;
    case 'colors':
      expTexelView = TexelView.fromTexelsAsColors(format, (_coords) => ({ R: -1 }));
      break;
  }

  const result = await textureContentIsOKByT2B(
    t,
    { texture },
    size,
    { expTexelView },
    { maxDiffULPsForNormFormat: 0 }
  );
  t.expectOK(result);
});