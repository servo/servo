/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'pack4xI8' builtin function

@const fn pack4xI8(e: vec4<i32>) -> u32
Pack the lower 8 bits of each component of e into a u32 value and drop all the unused bits.
Component e[i] of the input is mapped to bits (8 * i) through (8 * (i + 7)) of the result.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { u32, toVector, i32, Type } from '../../../../../util/conversion.js';

import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

g.test('basic').
specURL('https://www.w3.org/TR/WGSL/#pack4xI8-builtin').
desc(
  `
@const fn pack4xI8(e: vec4<i32>) -> u32
  `
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cfg = t.params;

  const pack4xI8 = (vals) => {
    const result = new Uint32Array(1);
    for (let i = 0; i < 4; ++i) {
      result[0] |= (vals[i] & 0xff) << i * 8;
    }
    return result[0];
  };

  const testInputs = [
  [0, 0, 0, 0],
  [1, 2, 3, 4],
  [-1, 2, 3, 4],
  [1, -2, 3, 4],
  [1, 2, -3, 4],
  [1, 2, 3, -4],
  [-1, -2, 3, 4],
  [-1, 2, -3, 4],
  [-1, 2, 3, -4],
  [1, -2, -3, 4],
  [1, -2, 3, -4],
  [1, 2, -3, -4],
  [-1, -2, -3, 4],
  [-1, -2, 3, -4],
  [-1, 2, -3, -4],
  [1, -2, -3, -4],
  [-1, -2, -3, -4],
  [127, 128, -128, -129],
  [128, 128, -128, -128],
  [32767, 32768, -32768, -32769]];


  const makeCase = (vals) => {
    return { input: [toVector(vals, i32)], expected: u32(pack4xI8(vals)) };
  };
  const cases = testInputs.flatMap((v) => {
    return [makeCase(v)];
  });

  await run(t, builtin('pack4xI8'), [Type.vec4i], Type.u32, cfg, cases);
});