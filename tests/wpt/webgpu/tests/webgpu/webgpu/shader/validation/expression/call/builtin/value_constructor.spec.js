/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for constructor built-in functions.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import {
  isAbstractType,
  isConvertible,
  isFloatType,
  MatrixType,
  ScalarType,
  scalarTypeOf,
  Type,
  VectorType } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kScalarTypes = ['bool', 'i32', 'u32', 'f32', 'f16'];

g.test('scalar_zero_value').
desc('Tests zero value scalar constructors').
params((u) => u.combine('type', kScalarTypes)).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const enable = t.params.type === 'f16' ? 'enable f16;' : '';
  const code = `${enable}
    const x : ${t.params.type} = ${t.params.type}();
    const_assert x == ${t.params.type}(0);`;
  t.expectCompileResult(true, code);
});

g.test('scalar_value').
desc('Tests scalar value constructors').
params((u) =>
u.
combine('type', kScalarTypes).
combine('value_type', [...kScalarTypes, 'vec2u', 'S', 'array<u32, 2>'])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16' || t.params.value_type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const enable = t.params.type === 'f16' || t.params.value_type === 'f16' ? 'enable f16;' : '';
  const code = `${enable}
    const x : ${t.params.type} = ${t.params.type}(${t.params.value_type}());`;
  t.expectCompileResult(kScalarTypes.includes(t.params.value_type), code);
});

g.test('vector_zero_value').
desc('Tests zero value vector constructors').
params((u) =>
u.
combine('type', [...kScalarTypes, 'abstract-int', 'abstract-float']).
beginSubcases().
combine('size', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const abstract = t.params.type === 'abstract-int' || t.params.type === 'abstract-float';
  const param = abstract ? '' : `<${t.params.type}>`;
  const decl = `vec${t.params.size}${param}`;
  const enable = t.params.type === 'f16' ? 'enable f16;' : '';
  const comparison = abstract ? '0' : `${t.params.type}(0)`;
  let code = `${enable}
    const x ${abstract ? '' : `: ${decl}`} = ${decl}();\n`;
  for (let i = 0; i < t.params.size; i++) {
    code += `const_assert x[${i}] == ${comparison};\n`;
  }
  t.expectCompileResult(true, code);
});

g.test('vector_splat').
desc('Test vector splat constructors').
params((u) =>
u.
combine('type', [
'bool',
'i32',
'u32',
'f32',
'f16',
'abstract-int',
'abstract-float']
).
combine('ele_type', [
'bool',
'i32',
'u32',
'f32',
'f16',
'abstract-int',
'abstract-float',
'mat2x2f',
'mat3x3h',
'vec2i',
'vec3f']
).
beginSubcases().
combine('size', [2, 3, 4])
).
beforeAllSubcases((t) => {
  const ty = Type[t.params.type];
  const eleTy = Type[t.params.ele_type];
  if (ty.requiresF16() || eleTy.requiresF16()) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const eleTy = Type[t.params.ele_type];
  const abstract = t.params.type === 'abstract-int' || t.params.type === 'abstract-float';
  const param = abstract ? '' : `<${t.params.type}>`;
  const decl = `vec${t.params.size}${param}`;
  const enable = t.params.type === 'f16' || t.params.ele_type === 'f16' ? 'enable f16;' : '';
  const eleValue = eleTy.create(1).wgsl();
  const valueCall = decl;
  const code = `${enable}
    const x ${abstract ? '' : `: ${decl}`} = ${valueCall}(${eleValue});`;
  const ty = Type[t.params.type];
  const expect =
  eleTy instanceof ScalarType && (isConvertible(eleTy, ty) || isAbstractType(ty)) ||
  eleTy instanceof VectorType && eleTy.width === t.params.size;
  t.expectCompileResult(expect, code);
});

g.test('vector_copy').
desc('Test vector copy constructors').
params((u) =>
u.
combine('decl_type', [
'bool',
'i32',
'u32',
'f32',
'f16',
'abstract-int',
'abstract-float']
).
combine('value_type', [
'bool',
'i32',
'u32',
'f32',
'f16',
'abstract-int',
'abstract-float']
).
beginSubcases().
combine('decl_size', [2, 3, 4]).
combine('value_size', [2, 3, 4])
).
beforeAllSubcases((t) => {
  const ty = Type[t.params.decl_type];
  const eleTy = Type[t.params.value_type];
  if (ty.requiresF16() || eleTy.requiresF16()) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const declTy = Type[t.params.decl_type];
  const valueTy = Type[t.params.value_type];
  const valueVecTy = Type['vec'](t.params.value_size, valueTy);
  const enable = declTy.requiresF16() || valueTy.requiresF16() ? 'enable f16;' : '';
  const decl = `vec${t.params.decl_size}<${t.params.decl_type}>`;
  const ctor = `vec${t.params.decl_size}${
  isAbstractType(declTy) ? '' : `<${t.params.decl_type}>`
  }`;
  const code = `${enable}
    const x ${isAbstractType(declTy) ? '' : `: ${decl}`} = ${ctor}(${valueVecTy.
  create(1).
  wgsl()});`;

  t.expectCompileResult(t.params.decl_size === t.params.value_size, code);
});

g.test('vector_elementwise').
desc('Test element-wise vector constructors').
params((u) =>
u.
combine('type', [
'bool',
'i32',
'u32',
'f32',
'f16',
'abstract-int',
'abstract-float']
).
combine('ele_type', [
'bool',
'i32',
'u32',
'f32',
'f16',
'abstract-int',
'abstract-float',
'mat2x2f',
'mat3x3h',
'vec2i',
'vec3f']
).
beginSubcases().
combine('size', [2, 3, 4]).
combine('num_eles', [2, 3, 4, 5]).
combine('full_type', [true, false])
).
beforeAllSubcases((t) => {
  const ty = Type[t.params.type];
  const eleTy = Type[t.params.ele_type];
  if (ty.requiresF16() || eleTy.requiresF16()) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const eleTy = Type[t.params.ele_type];
  const abstract = t.params.type === 'abstract-int' || t.params.type === 'abstract-float';
  const param = abstract ? '' : `<${t.params.type}>`;
  const decl = `vec${t.params.size}${param}`;
  const enable = t.params.type === 'f16' || t.params.ele_type === 'f16' ? 'enable f16;' : '';
  const eleValue = eleTy.create(1).wgsl();
  const valueCall = t.params.full_type ? decl : `vec${t.params.size}`;
  let code = `${enable}
    const x ${abstract ? '' : `: ${decl}`} = ${valueCall}(`;
  for (let i = 0; i < t.params.num_eles; i++) {
    code += `${eleValue},`;
  }
  code += `);`;
  const ty = Type[t.params.type];
  // WGSL requires:
  // * number of elements match
  // * element types match (or auto convert, vector special case)
  //   * abstract decl works because it is untyped and inferred as a different type
  const num_eles =
  eleTy instanceof VectorType ? t.params.num_eles * eleTy.width : t.params.num_eles;
  const expect =
  !(eleTy instanceof MatrixType) &&
  t.params.size === num_eles && (
  isConvertible(scalarTypeOf(eleTy), ty) ||
  t.params.type === 'abstract-int' ||
  t.params.type === 'abstract-float');
  t.expectCompileResult(expect, code);
});

g.test('vector_mixed').
desc('Test vector constructors with mixed elements and vectors').
params((u) =>
u.
combine('type', [
'bool',
'i32',
'u32',
'f32',
'f16',
'abstract-int',
'abstract-float']
).
combine('ele_type', [
'bool',
'i32',
'u32',
'f32',
'f16',
'abstract-int',
'abstract-float']
).
beginSubcases().
combine('size', [3, 4]).
combine('num_eles', [3, 4, 5]).
combine('full_type', [true, false])
).
beforeAllSubcases((t) => {
  const ty = Type[t.params.type];
  const eleTy = Type[t.params.ele_type];
  if (ty.requiresF16() || eleTy.requiresF16()) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const eleTy = Type[t.params.ele_type];
  const abstract = t.params.type === 'abstract-int' || t.params.type === 'abstract-float';
  const param = abstract ? '' : `<${t.params.type}>`;
  const decl = `vec${t.params.size}${param}`;
  const enable = t.params.type === 'f16' || t.params.ele_type === 'f16' ? 'enable f16;' : '';
  const v = eleTy.create(1).wgsl();
  const call = t.params.full_type ? decl : `vec${t.params.size}`;
  let code = `${enable}\n`;

  switch (t.params.num_eles) {
    case 3:
      code += `const x1 ${abstract ? '' : `: ${decl}`} = ${call}(${v}, vec2(${v}, ${v}));\n`;
      code += `const x2 ${abstract ? '' : `: ${decl}`} = ${call}(vec2(${v}, ${v}), ${v});\n`;
      break;
    case 4:
      code += `const x1 ${
      abstract ? '' : `: ${decl}`
      } = ${call}(${v}, vec2(${v}, ${v}), ${v});\n`;
      code += `const x2 ${
      abstract ? '' : `: ${decl}`
      } = ${call}(${v}, ${v}, vec2(${v}, ${v}));\n`;
      code += `const x3 ${
      abstract ? '' : `: ${decl}`
      } = ${call}(vec2(${v}, ${v}), ${v}, ${v});\n`;
      code += `const x4 ${
      abstract ? '' : `: ${decl}`
      } = ${call}(vec3(${v}, ${v}, ${v}), ${v});\n`;
      code += `const x5 ${
      abstract ? '' : `: ${decl}`
      } = ${call}(${v}, vec3(${v}, ${v}, ${v}));\n`;
      break;
    case 5:
      // This case is always invalid so try a few only.
      code += `const x1 ${
      abstract ? '' : `: ${decl}`
      } = ${call}(${v}, vec3(${v}, ${v}), ${v});\n`;
      code += `const x1 ${abstract ? '' : `: ${decl}`} = ${call}(${v}, vec4(${v}}), ${v});\n`;
      break;
  }
  const ty = Type[t.params.type];
  // WGSL requires:
  // * number of elements match (in total, not parameters)
  // * element types match (or auto convert)
  //   * abstract decl works because it is untyped and inferred as a different type
  const expect =
  t.params.size === t.params.num_eles && (
  isConvertible(eleTy, ty) ||
  t.params.type === 'abstract-int' ||
  t.params.type === 'abstract-float');
  t.expectCompileResult(expect, code);
});

g.test('matrix_zero_value').
desc('Tests zero value matrix constructors').
params((u) =>
u.
combine('type', ['f32', 'f16']).
beginSubcases().
combine('rows', [2, 3, 4]).
combine('cols', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const decl = `mat${t.params.cols}x${t.params.rows}<${t.params.type}>`;
  const enable = t.params.type === 'f16' ? 'enable f16;' : '';
  let code = `${enable}
    const x : ${decl} = ${decl}();\n`;
  for (let c = 0; c < t.params.cols; c++) {
    for (let r = 0; r < t.params.rows; r++) {
      code += `const_assert x[${c}][${r}] == ${t.params.type}(0);\n`;
    }
  }
  t.expectCompileResult(true, code);
});

g.test('matrix_copy').
desc('Test matrix copy constructors').
params((u) =>
u.
combine('type1', ['f16', 'f32', 'abstract-float']).
combine('type2', ['f16', 'f32', 'abstract-float']).
beginSubcases().
combine('c1', [2, 3, 4]).
combine('r1', [2, 3, 4]).
combine('c2', [2, 3, 4]).
combine('r2', [2, 3, 4])
).
beforeAllSubcases((t) => {
  const t1 = Type[t.params.type1];
  const t2 = Type[t.params.type2];
  if (t1.requiresF16() || t2.requiresF16()) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const t1 = Type[t.params.type1];
  const t2 = Type[t.params.type2];
  const m2 = Type['mat'](t.params.c2, t.params.r2, t2);
  const enable = t1.requiresF16() || t2.requiresF16() ? 'enable f16;' : '';
  const decl = `mat${t.params.c1}x${t.params.r1}<${t.params.type1}>`;
  const call = `mat${t.params.c1}x${t.params.r1}${
  isAbstractType(t1) ? '' : `<${t.params.type1}>`
  }`;
  const code = `${enable}
    const m ${isAbstractType(t1) ? '' : `: ${decl}`} = ${call}(${m2.create(0).wgsl()});`;
  t.expectCompileResult(t.params.c1 === t.params.c2 && t.params.r1 === t.params.r2, code);
});

g.test('matrix_column').
desc('Test matrix column constructors').
params((u) =>
u.
combine('type1', ['f16', 'f32', 'abstract-float']).
combine('type2', ['f16', 'f32', 'abstract-float', 'i32', 'u32', 'bool']).
beginSubcases().
combine('c1', [2, 3, 4]).
combine('r1', [2, 3, 4]).
combine('c2', [2, 3, 4]).
combine('r2', [2, 3, 4])
).
beforeAllSubcases((t) => {
  const t1 = Type[t.params.type1];
  const t2 = Type[t.params.type2];
  if (t1.requiresF16() || t2.requiresF16()) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const t1 = Type[t.params.type1];
  const t2 = Type[t.params.type2];
  const enable = t1.requiresF16() || t2.requiresF16() ? 'enable f16;' : '';
  const vecTy2 = Type['vec'](t.params.r2, t2);
  let values = ``;
  for (let i = 0; i < t.params.c2; i++) {
    values += `${vecTy2.create(1).wgsl()},`;
  }
  const decl = `mat${t.params.c1}x${t.params.r1}<${t.params.type1}>`;
  const call = `mat${t.params.c1}x${t.params.r1}${
  isAbstractType(t1) ? '' : `<${t.params.type1}>`
  }`;
  const code = `${enable}
    const m ${isAbstractType(t1) ? '' : `: ${decl}`} = ${call}(${values});`;
  const expect =
  isFloatType(t2) &&
  t.params.c1 === t.params.c2 &&
  t.params.r1 === t.params.r2 && (
  t1 === t2 || isAbstractType(t1) || isAbstractType(t2));
  t.expectCompileResult(expect, code);
});

g.test('matrix_elementwise').
desc('Test matrix element-wise constructors').
params((u) =>
u.
combine('type1', ['f16', 'f32', 'abstract-float']).
combine('type2', ['f16', 'f32', 'abstract-float', 'i32', 'u32', 'bool']).
beginSubcases().
combine('c1', [2, 3, 4]).
combine('r1', [2, 3, 4]).
combine('c2', [2, 3, 4]).
combine('r2', [2, 3, 4])
).
beforeAllSubcases((t) => {
  const t1 = Type[t.params.type1];
  const t2 = Type[t.params.type2];
  if (t1.requiresF16() || t2.requiresF16()) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const t1 = Type[t.params.type1];
  const t2 = Type[t.params.type2];
  const enable = t1.requiresF16() || t2.requiresF16() ? 'enable f16;' : '';
  let values = ``;
  for (let i = 0; i < t.params.c2 * t.params.r2; i++) {
    values += `${t2.create(1).wgsl()},`;
  }
  const decl = `mat${t.params.c1}x${t.params.r1}<${t.params.type1}>`;
  const call = `mat${t.params.c1}x${t.params.r1}${
  isAbstractType(t1) ? '' : `<${t.params.type1}>`
  }`;
  const code = `${enable}
    const m ${isAbstractType(t1) ? '' : `: ${decl}`} = ${call}(${values});`;
  const expect =
  isFloatType(t2) &&
  t.params.c1 * t.params.r1 === t.params.c2 * t.params.r2 && (
  t1 === t2 || isAbstractType(t1) || isAbstractType(t2));
  t.expectCompileResult(expect, code);
});








const kArrayCases = {
  i32: {
    element: 'i32',
    size: 4,
    valid: true,
    values: '1,2,3,4'
  },
  f32: {
    element: 'f32',
    size: 1,
    valid: true,
    values: '0'
  },
  u32: {
    element: 'u32',
    size: 2,
    valid: true,
    values: '2,4'
  },
  valid_array: {
    element: 'array<u32, 2>',
    size: 2,
    valid: true,
    values: 'array(0,1), array(2,3)'
  },
  invalid_rta: {
    element: 'u32',
    size: '',
    valid: false,
    values: '0'
  },
  invalid_override_array: {
    element: 'u32',
    size: 'o',
    valid: false,
    values: '1'
  },
  valid_struct: {
    element: 'valid_S',
    size: 1,
    valid: true,
    values: 'valid_S(0)'
  },
  invalid_struct: {
    element: 'invalid_S',
    size: 1,
    valid: false,
    values: 'array(0)'
  },
  invalid_atomic: {
    element: 'atomic<u32>',
    size: 1,
    valid: false,
    values: '0'
  }
};

g.test('array_zero_value').
desc('Tests zero value array constructors').
params((u) => u.combine('case', keysOf(kArrayCases))).
fn((t) => {
  const testcase = kArrayCases[t.params.case];
  const decl = `array<${testcase.element}, ${testcase.size}>`;
  const code = `override o : i32 = 1;
    struct valid_S {
      x : u32
    }
    struct invalid_S {
      x : array<u32>
    }
    const x : ${decl} = ${decl}();`;
  t.expectCompileResult(testcase.valid, code);
});

g.test('array_value').
desc('Tests array value constructor').
params((u) => u.combine('case', keysOf(kArrayCases))).
fn((t) => {
  const testcase = kArrayCases[t.params.case];
  const decl = `array<${testcase.element}, ${testcase.size}>`;
  const code = `override o : i32 = 1;
    struct valid_S {
      x : u32
    }
    struct invalid_S {
      x : array<u32>
    }
    const x : ${decl} = ${decl}(${testcase.values});`;
  t.expectCompileResult(testcase.valid, code);
});

const kStructCases = {
  i32: {
    name: 'S',
    decls: `struct S { x : u32 }`,
    valid: true,
    values: '0'
  },
  f32x2: {
    name: 'S',
    decls: `struct S { x : f32, y : f32 }`,
    valid: true,
    values: '0,1'
  },
  vec3u: {
    name: 'S',
    decls: `struct S { x : vec3u }`,
    valid: true,
    values: 'vec3()'
  },
  valid_array: {
    name: 'S',
    decls: `struct S { x : array<u32, 2> }`,
    valid: true,
    values: 'array(1,2)'
  },
  runtime_array: {
    name: 'S',
    decls: `struct S { x : array<u32> }`,
    valid: false,
    values: 'array(0)'
  },
  atomic: {
    name: 'S',
    decls: `struct S { x : atomic<u32> }`,
    valid: false,
    values: '0'
  },
  struct: {
    name: 'S',
    decls: `struct S {
      x : T
    }
    struct T {
      x : u32
    }`,
    valid: true,
    values: 'T(0)'
  },
  many_members: {
    name: 'S',
    decls: `struct S {
      a : bool,
      b : u32,
      c : i32,
      d : vec4f,
    }`,
    valid: true,
    values: 'false, 1u, 32i, vec4f(1.0f)'
  }
};

g.test('struct_zero_value').
desc('Tests zero value struct constructors').
params((u) => u.combine('case', keysOf(kStructCases))).
fn((t) => {
  const testcase = kStructCases[t.params.case];
  const code = `
    ${testcase.decls}
    const x : ${testcase.name} = ${testcase.name}();`;
  t.expectCompileResult(testcase.valid, code);
});

g.test('struct_value').
desc('Tests struct value constructors').
params((u) => u.combine('case', keysOf(kStructCases))).
fn((t) => {
  const testcase = kStructCases[t.params.case];
  const code = `
    ${testcase.decls}
    const x : ${testcase.name} = ${testcase.name}(${testcase.values});`;
  t.expectCompileResult(testcase.valid, code);
});

const kConstructors = {
  u32_0: 'u32()',
  i32_0: 'i32()',
  bool_0: 'bool()',
  f32_0: 'f32()',
  f16_0: 'f16()',
  vec2_0: 'vec2()',
  vec3_0: 'vec3()',
  vec4_0: 'vec4()',
  mat2x2_0: 'mat2x2f()',
  mat2x3_0: 'mat2x3f()',
  mat2x4_0: 'mat2x4f()',
  mat3x2_0: 'mat3x2f()',
  mat3x3_0: 'mat3x3f()',
  mat3x4_0: 'mat3x4f()',
  mat4x2_0_f16: 'mat4x2h()',
  mat4x3_0_f16: 'mat4x3h()',
  mat4x4_0_f16: 'mat4x4h()',
  S_0: 'S()',
  array_0: 'array<u32, 4>()',
  u32: 'u32(1)',
  i32: 'i32(1)',
  bool: 'bool(true)',
  f32: 'f32(1)',
  f16: 'f16(1)',
  vec2f: 'vec2<f32>(1)',
  vec3_f16: 'vec3<f16>(1)',
  vec4: 'vec4(1)',
  mat2x2: 'mat2x2f(1,1,1,1)',
  mat2x3: 'mat2x3f(1,1,1,1,1,1)',
  mat2x4: 'mat2x4f(1,1,1,1,1,1,1,1)',
  mat3x2_f16: 'mat3x2<f16>(vec2h(),vec2h(),vec2h())',
  mat3x3_f16: 'mat3x3<f16>(vec3h(),vec3h(),vec3h())',
  mat3x4_f16: 'mat3x4<f16>(vec4h(),vec4h(),vec4h())',
  mat4x2: 'mat4x2(vec2(),vec2(),vec2(),vec2())',
  mat4x3: 'mat4x3(vec3(),vec3(),vec3(),vec3())',
  mat4x4: 'mat4x4(vec4(),vec4(),vec4(),vec4())',
  S: 'S(1,1)',
  array_abs: 'array(1,2,3)',
  array: 'array<u32, 4>(1,2,3,4)'
};

g.test('must_use').
desc('Tests that value constructors must be used').
params((u) => u.combine('ctor', keysOf(kConstructors)).combine('use', [true, false])).
beforeAllSubcases((t) => {
  if (t.params.ctor.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const code = `
    ${t.params.ctor.includes('f16') ? 'enable f16;' : ''}
    struct S {
      x : u32,
      y : f32,
    }
    fn foo() {
      ${t.params.use ? '_ =' : ''} ${kConstructors[t.params.ctor]};
    }`;
  t.expectCompileResult(t.params.use, code);
});

g.test('partial_eval').
desc('Tests that mixed runtime and early eval expressions catch errors').
params((u) =>
u.
combine('eleTy', ['i32', 'u32']).
combine('compTy', ['array', 'vec2', 'vec3', 'vec4', 'S']).
combine('stage', ['constant', 'runtime']).
beginSubcases().
expandWithParams((t) => {
  const cases = [];
  switch (t.compTy) {
    case 'array':
      cases.push({ numEles: 2, index: 0 });
      cases.push({ numEles: 2, index: 1 });
      cases.push({ numEles: 3, index: 0 });
      cases.push({ numEles: 3, index: 1 });
      cases.push({ numEles: 3, index: 2 });
      break;
    case 'vec2':
      cases.push({ numEles: 2, index: 0 });
      cases.push({ numEles: 2, index: 1 });
      break;
    case 'vec3':
      cases.push({ numEles: 3, index: 0 });
      cases.push({ numEles: 3, index: 1 });
      cases.push({ numEles: 3, index: 2 });
      break;
    case 'vec4':
      cases.push({ numEles: 4, index: 0 });
      cases.push({ numEles: 4, index: 1 });
      cases.push({ numEles: 4, index: 2 });
      cases.push({ numEles: 4, index: 3 });
      break;
    case 'S':
      cases.push({ numEles: 2, index: 0 });
      cases.push({ numEles: 2, index: 1 });
      break;
  }
  return cases;
})
).
fn((t) => {
  const eleTy = Type['abstract-int'];
  const value = t.params.eleTy === 'i32' ? 0xfffffffff : -1;
  let compParams = '';
  for (let i = 0; i < t.params.numEles; i++) {
    if (t.params.index === i) {
      switch (t.params.stage) {
        case 'constant':
          compParams += `${eleTy.create(value).wgsl()}, `;
          break;
        case 'runtime':
          compParams += `v, `;
          break;
      }
    } else {
      compParams += `v, `;
    }
  }
  const wgsl = `
struct S {
  x : ${t.params.eleTy},
  y : ${t.params.eleTy},
}

fn foo() {
  var v : ${t.params.eleTy};
  let tmp = ${t.params.compTy}(${compParams});
}`;

  const shader_error = t.params.stage === 'constant';
  t.expectCompileResult(!shader_error, wgsl);
});