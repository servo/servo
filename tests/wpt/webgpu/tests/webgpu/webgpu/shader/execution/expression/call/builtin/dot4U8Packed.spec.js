/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'dot4U8Packed' builtin function

@const fn dot4U8Packed(e1: u32 ,e2: u32) -> u32
e1 and e2 are interpreted as vectors with four 8-bit unsigned integer components. Return the
unsigned integer dot product of these two vectors.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type, u32 } from '../../../../../util/conversion.js';

import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

g.test('basic').
specURL('https://www.w3.org/TR/WGSL/#dot4U8Packed-builtin').
desc(
  `
@const fn dot4U8Packed(e1: u32, e2: u32) -> u32
  `
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cfg = t.params;

  const dot4U8Packed = (e1, e2) => {
    let result = 0;
    for (let i = 0; i < 4; ++i) {
      const e1_i = e1 >> i * 8 & 0xff;
      const e2_i = e2 >> i * 8 & 0xff;
      result += e1_i * e2_i;
    }
    return result;
  };

  const testInputs = [
  // dot({0, 0, 0, 0}, {0, 0, 0, 0})
  [0, 0],
  // dot({255u, 255u, 255u, 255u}, {255u, 255u, 255u, 255u})
  [0xffffffff, 0xffffffff],
  // dot({1u, 2u, 3u, 4u}, {5u, 6u, 7u, 8u})
  [0x01020304, 0x05060708],
  // dot({120u, 90u, 60u, 30u}, {50u, 100u, 150u, 200u})
  [0x785a3c1e, 0x326496c8]];


  const makeCase = (x, y) => {
    return { input: [u32(x), u32(y)], expected: u32(dot4U8Packed(x, y)) };
  };
  const cases = testInputs.flatMap((v) => {
    return [makeCase(...v)];
  });

  await run(t, builtin('dot4U8Packed'), [Type.u32, Type.u32], Type.u32, cfg, cases);
});