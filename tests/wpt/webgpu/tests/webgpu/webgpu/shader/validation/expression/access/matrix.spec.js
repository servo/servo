/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for matrix access expressions

* Index type
* Result type
* Early-evaluation errors
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../common/util/data_tables.js';
import { Type } from '../../../../util/conversion.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('index_type').
desc('Tests valid index types for array access expressions').
params((u) =>
u.combine('type', [
'bool',
'u32',
'i32',
'abstract-int',
'f32',
'f16',
'abstract-float',
'vec2i']
)
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const ty = Type[t.params.type];
  const enable = ty.requiresF16() ? 'enable f16;' : '';
  const code = `${enable}
    fn foo() {
      var x = mat2x2(1,2,3,4);
      let tmp = x[${ty.create(0).wgsl()}];
    }`;
  const expect =
  t.params.type === 'i32' || t.params.type === 'u32' || t.params.type === 'abstract-int';
  t.expectCompileResult(expect, code);
});

g.test('result_type').
desc('Tests that correct result type is produced for an access expression').
params((u) =>
u.
combine('element', ['f16', 'f32']).
combine('columns', [2, 3, 4]).
beginSubcases().
combine('rows', [2, 3, 4]).
combine('decl', ['function', 'module'])
).
beforeAllSubcases((t) => {
  if (t.params.element === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const enable = t.params.element === 'f16' ? 'enable f16;' : '';
  const scalarTy = Type[t.params.element];
  const vectorTy = Type['vec'](t.params.rows, scalarTy);
  const matrixTy = Type['mat'](t.params.columns, t.params.rows, scalarTy);
  const module_decl =
  t.params.decl === 'module' ?
  `@group(0) @binding(0) var<storage> x : ${matrixTy.toString()};` :
  ``;
  const function_decl = t.params.decl === 'module' ? `` : `var x : ${matrixTy.toString()};`;
  const code = `${enable}
    ${module_decl}
    fn foo() {
      ${function_decl}
      let tmp1 : ${vectorTy.toString()} = x[0];
      let tmp2 : ${vectorTy.toString()} = x[1];
    }`;
  t.expectCompileResult(true, code);
});








const kOutOfBoundsCases = {
  const_module_in_bounds: {
    code: `const x = mat2x2(1,2,3,4)[0];`,
    result: true
  },
  const_module_oob_neg: {
    code: `const x = mat2x2(1,2,3,4)[-1];`,
    result: false
  },
  const_module_oob_pos: {
    code: `const x = mat2x2(1,2,3,4)[2];`,
    result: false
  },
  const_func_in_bounds: {
    code: `fn foo() {
      const x = mat2x2(1,2,3,4)[0];
    }`,
    result: true
  },
  const_func_oob_neg: {
    code: `fn foo {
      const x = mat2x2(1,2,3,4)[-1];
    }`,
    result: false
  },
  const_func_oob_pos: {
    code: `fn foo {
      const x = mat2x2(1,2,3,4)[2];
    }`,
    result: false
  },
  override_in_bounds: {
    code: `override x : i32;
    fn y() -> u32 {
      let tmp = mat2x2(1,2,3,4)[x];
      return 0;
    }`,
    result: true,
    pipeline: true,
    value: 0
  },
  override_oob_neg: {
    code: `override x : i32;
    fn y() -> u32 {
      let tmp = mat2x2(1,2,3,4)[x];
      return 0;
    }`,
    result: false,
    pipeline: true,
    value: -1
  },
  override_oob_pos: {
    code: `override x : i32;
    fn y() -> u32 {
      let tmp = mat2x2(1,2,3,4)[x];
      return 0;
    }`,
    result: false,
    pipeline: true,
    value: 2
  },
  runtime_in_bounds: {
    code: `fn foo() {
      let idx = 0;
      let x = mat2x2(1,2,3,4)[idx];
    }`,
    result: true
  },
  runtime_oob_neg: {
    code: `fn foo() {
      let idx = -1;
      let x = mat2x2(1,2,3,4)[idx];
    }`,
    result: true
  },
  runtime_oob_pos: {
    code: `fn foo() {
      let idx = 3;
      let x = mat2x2(1,2,3,4)[idx];
    }`,
    result: true
  },
  runtime_array_const_oob_neg: {
    code: `@group(0) @binding(0) var<storage> x : mat2x2<f32>;
    fn y() -> u32 {
      let tmp = x[-1];
      return 0;
    }`,
    result: false
  },
  runtime_array_override_oob_neg: {
    code: `@group(0) @binding(0) var<storage> v : mat2x2<f32>;
    override x : i32;
    fn y() -> u32 {
      let tmp = v[x];
      return 0;
    }`,
    result: false,
    pipeline: true,
    value: -1
  }
};

g.test('early_eval_errors').
desc('Tests early evaluation errors for out-of-bounds indexing').
params((u) => u.combine('case', keysOf(kOutOfBoundsCases))).
fn((t) => {
  const testcase = kOutOfBoundsCases[t.params.case];
  if (testcase.pipeline) {
    const v = testcase.value ?? 0;
    t.expectPipelineResult({
      expectedResult: testcase.result,
      code: testcase.code,
      constants: { x: v },
      reference: ['y()']
    });
  } else {
    t.expectCompileResult(testcase.result, testcase.code);
  }
});

g.test('abstract_matrix_concrete_index').
desc('Tests that a concrete index type on an abstract array remains abstract').
fn((t) => {
  const code = `
    const idx = 0i;
    const_assert mat2x2(1.11001100110011008404,1,1,1)[0i][0i] == 1.11001100110011008404;`;
  t.expectCompileResult(true, code);
});