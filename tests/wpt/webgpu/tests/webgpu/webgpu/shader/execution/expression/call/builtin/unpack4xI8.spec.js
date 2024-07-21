/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'unpack4xI8' builtin function

@const fn unpack4xI8(e: u32) -> vec4<i32>
e is interpreted as a vector with four 8-bit signed integer components. Unpack e into a vec4<i32>
with sign extension.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { u32, toVector, i32, Type } from '../../../../../util/conversion.js';

import { allInputSources, run } from '../../expression.js';

import { builtin } from './builtin.js';

export const g = makeTestGroup(GPUTest);

g.test('basic').
specURL('https://www.w3.org/TR/WGSL/#unpack4xI8-builtin').
desc(
  `
@const fn unpack4xI8(e: u32) -> vec4<i32>
  `
).
params((u) => u.combine('inputSource', allInputSources)).
fn(async (t) => {
  const cfg = t.params;

  const unpack4xI8 = (e) => {
    const result = [0, 0, 0, 0];
    for (let i = 0; i < 4; ++i) {
      let intValue = e >> 8 * i & 0xff;
      if (intValue > 127) {
        intValue -= 256;
      }
      result[i] = intValue;
    }
    return result;
  };

  const testInputs = [
  0, 0x01020304, 0xfcfdfeff, 0x040302ff, 0x0403fe01, 0x04fd0201, 0xfc030201, 0xfcfdfe01,
  0xfcfd02ff, 0xfc03feff, 0x04fdfeff, 0x0403feff, 0x04fd02ff, 0xfc0302ff, 0x04fdfe01,
  0xfc03fe01, 0xfcfd0201, 0x80817f7e];


  const makeCase = (e) => {
    return { input: [u32(e)], expected: toVector(unpack4xI8(e), i32) };
  };
  const cases = testInputs.flatMap((v) => {
    return [makeCase(v)];
  });

  await run(t, builtin('unpack4xI8'), [Type.u32], Type.vec4i, cfg, cases);
});