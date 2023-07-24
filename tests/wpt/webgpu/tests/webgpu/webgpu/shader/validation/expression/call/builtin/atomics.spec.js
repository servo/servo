/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Validation tests for atomic builtins.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kAtomicOps = {
  add: { src: 'atomicAdd(&a,1)' },
  sub: { src: 'atomicSub(&a,1)' },
  max: { src: 'atomicMax(&a,1)' },
  min: { src: 'atomicMin(&a,1)' },
  and: { src: 'atomicAnd(&a,1)' },
  or: { src: 'atomicOr(&a,1)' },
  xor: { src: 'atomicXor(&a,1)' },
  load: { src: 'atomicLoad(&a)' },
  store: { src: 'atomicStore(&a,1)' },
  exchange: { src: 'atomicExchange(&a,1)' },
  compareexchangeweak: { src: 'atomicCompareExchangeWeak(&a,1,1)' },
};

g.test('stage')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-rmw')
  .desc(
    `
Atomic built-in functions must not be used in a vertex shader stage.
`
  )
  .params(u =>
    u
      .combine('stage', ['fragment', 'vertex', 'compute']) //
      .combine('atomicOp', keysOf(kAtomicOps))
  )
  .fn(t => {
    const atomicOp = kAtomicOps[t.params.atomicOp].src;
    let code = `
@group(0) @binding(0) var<storage, read_write> a: atomic<i32>;
`;

    switch (t.params.stage) {
      case 'compute':
        code += `
@compute @workgroup_size(1,1,1) fn main() {
  ${atomicOp};
}`;
        break;

      case 'fragment':
        code += `
@fragment fn main() -> @location(0) vec4<f32> {
  ${atomicOp};
  return vec4<f32>();
}`;
        break;

      case 'vertex':
        code += `
@vertex fn vmain() -> @builtin(position) vec4<f32> {
  ${atomicOp};
  return vec4<f32>();
}`;
        break;
    }

    const pass = t.params.stage !== 'vertex';
    t.expectCompileResult(pass, code);
  });
