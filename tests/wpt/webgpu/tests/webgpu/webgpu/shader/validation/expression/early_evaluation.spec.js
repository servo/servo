/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests specific validation for early evaluation expressions
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);







const kCompositeCases = {
  const_scalar: {
    code: `let tmp = const_1e30 * const_1e30;`,
    stage: 'constant',
    valid: false
  },
  const_vector: {
    code: `let tmp = vec4(const_1e30) * vec4(const_1e30);`,
    stage: 'constant',
    valid: false
  },
  const_let_vector: {
    code: `let tmp = vec4(const_1e30) * vec4(vec3(const_1e30), let_1e30);`,
    stage: 'constant',
    valid: true
  },
  const_let_vector_comp: {
    code: `let tmp = vec2(const_1e30)[0] * vec2(const_1e30, let_1e30)[0];`,
    stage: 'constant',
    valid: true
  },
  const_let_array_comp: {
    code: `let tmp = array(const_1e30, const_1e30)[0] * array(const_1e30, let_1e30)[0];`,
    stage: 'constant',
    valid: true
  },
  const_let_struct_comp: {
    code: `let tmp = S(const_1e30, const_1e30).x * S(const_1e30, let_1e30).x;`,
    stage: 'constant',
    valid: true
  },
  const_let_matrix: {
    code: `let tmp = mat2x2(vec2(const_1e30), vec2(const_1e30)) * mat2x2(vec2(const_1e30), vec2(let_1e30));`,
    stage: 'constant',
    valid: true
  },
  const_let_matrix_vec: {
    code: `let tmp = mat2x2(vec2(const_1e30), vec2(const_1e30))[0] * mat2x2(vec2(const_1e30), vec2(let_1e30))[0];`,
    stage: 'constant',
    valid: true
  },
  const_let_matrix_comp: {
    code: `let tmp = mat2x2(vec2(const_1e30), vec2(const_1e30))[0].x * mat2x2(vec2(const_1e30), vec2(let_1e30))[0].x;`,
    stage: 'constant',
    valid: true
  },
  override_scalar: {
    code: `let tmp = override_1e30 * override_1e30;`,
    stage: 'override',
    valid: false
  },
  override_vector: {
    code: `let tmp = vec4(override_1e30) * vec4(override_1e30);`,
    stage: 'override',
    valid: false
  },
  override_let_vector: {
    code: `let tmp = vec4(override_1e30) * vec4(vec3(override_1e30), let_1e30);`,
    stage: 'override',
    valid: true
  },
  override_let_vector_comp: {
    code: `let tmp = vec2(override_1e30)[0] * vec2(override_1e30, let_1e30)[0];`,
    stage: 'override',
    valid: true
  },
  override_let_array_comp: {
    code: `let tmp = array(override_1e30, override_1e30)[0] * array(override_1e30, let_1e30)[0];`,
    stage: 'override',
    valid: true
  },
  override_let_struct_comp: {
    code: `let tmp = S(override_1e30, override_1e30).x * S(override_1e30, let_1e30).x;`,
    stage: 'override',
    valid: true
  },
  override_let_matrix: {
    code: `let tmp = mat2x2(vec2(override_1e30), vec2(override_1e30)) * mat2x2(vec2(override_1e30), vec2(let_1e30));`,
    stage: 'override',
    valid: true
  },
  override_let_matrix_vec: {
    code: `let tmp = mat2x2(vec2(override_1e30), vec2(override_1e30))[0] * mat2x2(vec2(override_1e30), vec2(let_1e30))[0];`,
    stage: 'override',
    valid: true
  },
  override_let_matrix_comp: {
    code: `let tmp = mat2x2(vec2(override_1e30), vec2(override_1e30))[0].x * mat2x2(vec2(override_1e30), vec2(let_1e30))[0].x;`,
    stage: 'override',
    valid: true
  }
};

g.test('composites').
desc('Validates that composites are either wholly evaluated or not at all').
params((u) => u.combine('case', keysOf(kCompositeCases))).
fn((t) => {
  const { code, stage, valid } = kCompositeCases[t.params.case];
  const wgsl = `
struct S {
  x : f32,
  y : f32,
}
const const_1e30 = f32(1e30);
override override_1e30 : f32;
fn foo() -> u32 {
  let let_1e30 = f32(1e30);
  ${code}
  return 0;
}`;

  if (stage === 'constant') {
    t.expectCompileResult(valid, wgsl);
  } else {
    const constants = {};
    constants['override_1e30'] = 1e30;
    t.expectPipelineResult({
      expectedResult: valid,
      code: wgsl,
      constants,
      reference: ['override_1e30', 'foo()']
    });
  }
});