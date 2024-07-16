/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for array access expressions

* Index type
* Result type
* Early-evaluation errors
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../common/util/data_tables.js';
import {
  Type,
  elementTypeOf,
  kConcreteNumericScalarsAndVectors,
  kAllBoolScalarsAndVectors } from
'../../../../util/conversion.js';
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
      var x = array(1,2,3);
      let tmp = x[${ty.create(0).wgsl()}];
    }`;
  const expect =
  t.params.type === 'i32' || t.params.type === 'u32' || t.params.type === 'abstract-int';
  t.expectCompileResult(expect, code);
});

const kTypes = objectsToRecord([
...kConcreteNumericScalarsAndVectors,
...kAllBoolScalarsAndVectors]
);
const kTypeKeys = keysOf(kTypes);

g.test('result_type').
desc('Tests that correct result type is produced for an access expression').
params((u) =>
u.
combine('type', kTypeKeys).
combine('elements', [0, 4]).
filter((t) => {
  const ty = kTypes[t.type];
  if (t.elements === 0) {
    if (elementTypeOf(ty) === Type.bool) {
      return false;
    }
  }
  return true;
})
).
beforeAllSubcases((t) => {
  const ty = kTypes[t.params.type];
  if (ty.requiresF16()) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const ty = kTypes[t.params.type];
  const enable = ty.requiresF16() ? 'enable f16;' : '';
  const arrayTy = Type['array'](t.params.elements, ty);
  const module_decl =
  t.params.elements === 0 ?
  `@group(0) @binding(0) var<storage> x : ${arrayTy.toString()};` :
  ``;
  const function_decl = t.params.elements === 0 ? `` : `var x : ${arrayTy.toString()};`;
  const code = `${enable}
    ${module_decl}
    fn foo() {
      ${function_decl}
      let tmp1 : ${ty.toString()} = x[0];
      let tmp2 : ${ty.toString()} = x[1];
      let tmp3 : ${ty.toString()} = x[2];
    }`;
  t.expectCompileResult(true, code);
});








const kOutOfBoundsCases = {
  const_module_in_bounds: {
    code: `const x = array(1,2,3)[0];`,
    result: true
  },
  const_module_oob_neg: {
    code: `const x = array(1,2,3)[-1];`,
    result: false
  },
  const_module_oob_pos: {
    code: `const x = array(1,2,3)[3];`,
    result: false
  },
  const_func_in_bounds: {
    code: `fn foo() {
      const x = array(1,2,3)[0];
    }`,
    result: true
  },
  const_func_oob_neg: {
    code: `fn foo {
      const x = array(1,2,3)[-1];
    }`,
    result: false
  },
  const_func_oob_pos: {
    code: `fn foo {
      const x = array(1,2,3)[3];
    }`,
    result: false
  },
  override_in_bounds: {
    code: `override x : i32;
    fn y() -> u32 {
      let tmp = array(1,2,3)[x];
      return 0;
    }`,
    result: true,
    pipeline: true,
    value: 0
  },
  override_oob_neg: {
    code: `override x : i32;
    fn y() -> u32 {
      let tmp = array(1,2,3)[x];
      return 0;
    }`,
    result: false,
    pipeline: true,
    value: -1
  },
  override_oob_pos: {
    code: `override x : i32;
    fn y() -> u32 {
      let tmp = array(1,2,3)[x];
      return 0;
    }`,
    result: false,
    pipeline: true,
    value: 3
  },
  runtime_in_bounds: {
    code: `fn foo() {
      let idx = 0;
      let x = array(1,2,3)[idx];
    }`,
    result: true
  },
  runtime_oob_neg: {
    code: `fn foo() {
      let idx = -1;
      let x = array(1,2,3)[idx];
    }`,
    result: true
  },
  runtime_oob_pos: {
    code: `fn foo() {
      let idx = 3;
      let x = array(1,2,3)[idx];
    }`,
    result: true
  },
  runtime_array_const_oob_neg: {
    code: `@group(0) @binding(0) var<storage> x : array<u32>;
    fn y() -> u32 {
      let tmp = x[-1];
      return 0;
    }`,
    result: false
  },
  runtime_array_override_oob_neg: {
    code: `@group(0) @binding(0) var<storage> v : array<u32>;
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

g.test('abstract_array_concrete_index').
desc('Tests that a concrete index type on an abstract array remains abstract').
fn((t) => {
  const code = `
    const idx = 0i;
    const_assert array(0xfffffffff,2,3)[idx] == 0xfffffffff;`;
  t.expectCompileResult(true, code);
});