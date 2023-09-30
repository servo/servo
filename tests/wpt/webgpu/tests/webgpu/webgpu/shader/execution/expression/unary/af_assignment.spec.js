/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for assignment of AbstractFloats
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import { kValue } from '../../../../util/constants.js';
import { abstractFloat, TypeAbstractFloat, TypeF16, TypeF32 } from '../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import {
  filteredF64Range,
  fullF64Range,
  isSubnormalNumberF64,
  reinterpretU64AsF64,
} from '../../../../util/math.js';
import { makeCaseCache } from '../case_cache.js';
import {
  abstractFloatShaderBuilder,
  basicExpressionBuilder,
  onlyConstInputSource,
  run,
} from '../expression.js';

function concrete_assignment() {
  return basicExpressionBuilder(value => `${value}`);
}

function abstract_assignment() {
  return abstractFloatShaderBuilder(value => `${value}`);
}

export const g = makeTestGroup(GPUTest);

export const d = makeCaseCache('unary/af_assignment', {
  abstract: () => {
    const inputs = [
      // Values that are useful for debugging the underlying framework/shader code, since it cannot be directly unit tested.
      0,
      0.5,
      0.5,
      1,
      -1,
      reinterpretU64AsF64(0x7000_0000_0000_0001n), // smallest magnitude negative subnormal with non-zero mantissa
      reinterpretU64AsF64(0x0000_0000_0000_0001n), // smallest magnitude positive subnormal with non-zero mantissa
      reinterpretU64AsF64(0x600a_aaaa_5555_5555n), // negative subnormal with obvious pattern
      reinterpretU64AsF64(0x000a_aaaa_5555_5555n), // positive subnormal with obvious pattern
      reinterpretU64AsF64(0x0010_0000_0000_0001n), // smallest magnitude negative normal with non-zero mantissa
      reinterpretU64AsF64(0x0010_0000_0000_0001n), // smallest magnitude positive normal with non-zero mantissa
      reinterpretU64AsF64(0xf555_5555_aaaa_aaaan), // negative normal with obvious pattern
      reinterpretU64AsF64(0x5555_5555_aaaa_aaaan), // positive normal with obvious pattern
      reinterpretU64AsF64(0xffef_ffff_ffff_ffffn), // largest magnitude negative normal
      reinterpretU64AsF64(0x7fef_ffff_ffff_ffffn), // largest magnitude positive normal
      // WebGPU implementation stressing values
      ...fullF64Range(),
    ];

    return inputs.map(f => {
      return {
        input: abstractFloat(f),
        expected: isSubnormalNumberF64(f) ? abstractFloat(0) : abstractFloat(f),
      };
    });
  },
  f32: () => {
    return filteredF64Range(kValue.f32.negative.min, kValue.f32.positive.max).map(f => {
      return { input: abstractFloat(f), expected: FP.f32.correctlyRoundedInterval(f) };
    });
  },
  f16: () => {
    return filteredF64Range(kValue.f16.negative.min, kValue.f16.positive.max).map(f => {
      return { input: abstractFloat(f), expected: FP.f16.correctlyRoundedInterval(f) };
    });
  },
});

g.test('abstract')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-conversion')
  .desc(
    `
testing that extracting abstract floats works
`
  )
  .params(u => u.combine('inputSource', onlyConstInputSource))
  .fn(async t => {
    const cases = await d.get('abstract');
    await run(t, abstract_assignment(), [TypeAbstractFloat], TypeAbstractFloat, t.params, cases, 1);
  });

g.test('f32')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-conversion')
  .desc(
    `
concretizing to f32
`
  )
  .params(u => u.combine('inputSource', onlyConstInputSource))
  .fn(async t => {
    const cases = await d.get('f32');
    await run(t, concrete_assignment(), [TypeAbstractFloat], TypeF32, t.params, cases);
  });

g.test('f16')
  .specURL('https://www.w3.org/TR/WGSL/#floating-point-conversion')
  .desc(
    `
concretizing to f16
`
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .params(u => u.combine('inputSource', onlyConstInputSource))
  .fn(async t => {
    const cases = await d.get('f16');
    await run(t, concrete_assignment(), [TypeAbstractFloat], TypeF16, t.params, cases);
  });
