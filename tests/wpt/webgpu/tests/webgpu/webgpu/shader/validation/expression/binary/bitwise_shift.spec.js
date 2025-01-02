/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for the bitwise shift binary expression operations
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../common/util/data_tables.js';
import {
  Type,
  kAllScalarsAndVectors,
  numElementsOf,
  scalarTypeOf } from
'../../../../util/conversion.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// Converts v to signed decimal number.
// Required because JS binary literals are always interpreted as unsigned numbers.
function signed(v) {
  return new Int32Array([v])[0];
}

// Return vector form of size `size` of input value `v`, or `v` if size is undefined.
function vectorize(v, size) {
  if (size !== undefined) {
    return `vec${size}(${v})`;
  }
  return v;
}

// A list of scalar and vector types.
const kScalarAndVectorTypes = objectsToRecord(kAllScalarsAndVectors);

g.test('scalar_vector').
desc(
  `
  Validates that scalar and vector expressions are only accepted when the LHS is an integer and the RHS is abstract or unsigned.
  `
).
params((u) =>
u.
combine('lhs', keysOf(kScalarAndVectorTypes)).
combine(
  'rhs',
  // Skip vec3 and vec4 on the RHS to keep the number of subcases down.
  keysOf(kScalarAndVectorTypes).filter(
    (value) => !(value.startsWith('vec3') || value.startsWith('vec4'))
  )
).
combine('compound_assignment', [false, true]).
beginSubcases().
combine('op', ['<<', '>>'])
).
beforeAllSubcases((t) => {
  if (
  scalarTypeOf(kScalarAndVectorTypes[t.params.lhs]) === Type.f16 ||
  scalarTypeOf(kScalarAndVectorTypes[t.params.rhs]) === Type.f16)
  {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const lhs = kScalarAndVectorTypes[t.params.lhs];
  const rhs = kScalarAndVectorTypes[t.params.rhs];
  const lhsElement = scalarTypeOf(lhs);
  const rhsElement = scalarTypeOf(rhs);
  const hasF16 = lhsElement === Type.f16 || rhsElement === Type.f16;
  const code = t.params.compound_assignment ?
  `
${hasF16 ? 'enable f16;' : ''}
fn f() {
  var foo = ${lhs.create(0).wgsl()};
  foo ${t.params.op}= ${rhs.create(0).wgsl()};
}
` :
  `
${hasF16 ? 'enable f16;' : ''}
const lhs = ${lhs.create(0).wgsl()};
const rhs = ${rhs.create(0).wgsl()};
const foo = lhs ${t.params.op} rhs;
`;

  // The LHS must be an integer, and the RHS must be an abstract/unsigned integer.
  // The vector widths must also match.
  const lhs_valid = [Type.abstractInt, Type.i32, Type.u32].includes(lhsElement);
  const rhs_valid = [Type.abstractInt, Type.u32].includes(rhsElement);
  const valid = lhs_valid && rhs_valid && numElementsOf(lhs) === numElementsOf(rhs);
  t.expectCompileResult(valid, code);
});







const kInvalidTypes = {
  mat2x2f: {
    expr: 'm',
    control: (e) => `u32(${e}[0][0])`
  },

  array: {
    expr: 'arr',
    control: (e) => `${e}[0]`
  },

  ptr: {
    expr: '(&u)',
    control: (e) => `*${e}`
  },

  atomic: {
    expr: 'a',
    control: (e) => `atomicLoad(&${e})`
  },

  texture: {
    expr: 't',
    control: (e) => `u32(textureLoad(${e}, vec2(), 0).x)`
  },

  sampler: {
    expr: 's',
    control: (e) => `u32(textureSampleLevel(t, ${e}, vec2(), 0).x)`
  },

  struct: {
    expr: 'str',
    control: (e) => `${e}.u`
  }
};

g.test('invalid_types').
desc(
  `
  Validates that expressions are never accepted for non-scalar and non-vector types.
  `
).
params((u) =>
u.
combine('op', ['<<', '>>']).
combine('type', keysOf(kInvalidTypes)).
combine('control', [true, false]).
beginSubcases()
).
fn((t) => {
  const type = kInvalidTypes[t.params.type];
  const expr = t.params.control ? type.control(type.expr) : type.expr;
  const code = `
@group(0) @binding(0) var t : texture_2d<f32>;
@group(0) @binding(1) var s : sampler;
@group(0) @binding(2) var<storage, read_write> a : atomic<u32>;

struct S { u : u32 }

var<private> u : u32;
var<private> m : mat2x2f;
var<private> arr : array<u32, 4>;
var<private> str : S;

@compute @workgroup_size(1)
fn main() {
  let foo = ${expr} ${t.params.op} ${expr};
}
`;

  t.expectCompileResult(t.params.control, code);
});

const kLeftShiftCases = [
// rhs >= bitwidth fails
{ lhs: `0u`, rhs: `31u`, pass: true },
{ lhs: `0u`, rhs: `32u`, pass: false },
{ lhs: `0u`, rhs: `33u`, pass: false },
{ lhs: `0u`, rhs: `1000u`, pass: false },
{ lhs: `0u`, rhs: `0xFFFFFFFFu`, pass: false },

{ lhs: `0i`, rhs: `31u`, pass: true },
{ lhs: `0i`, rhs: `32u`, pass: false },
{ lhs: `0i`, rhs: `33u`, pass: false },
{ lhs: `0i`, rhs: `1000u`, pass: false },
{ lhs: `0i`, rhs: `0xFFFFFFFFu`, pass: false },

// Signed overflow (sign change)
{ lhs: `${0b01000000000000000000000000000000}i`, rhs: `1u`, pass: false },
{ lhs: `${0b01111111111111111111111111111111}i`, rhs: `1u`, pass: false },
{ lhs: `${0b00000000000000000000000000000001}i`, rhs: `31u`, pass: false },
// Same cases should pass if lhs is unsigned
{ lhs: `${0b01000000000000000000000000000000}u`, rhs: `1u`, pass: true },
{ lhs: `${0b01111111111111111111111111111111}u`, rhs: `1u`, pass: true },
{ lhs: `${0b00000000000000000000000000000001}u`, rhs: `31u`, pass: true },

// Unsigned overflow
{ lhs: `${0b11000000000000000000000000000000}u`, rhs: `1u`, pass: false },
{ lhs: `${0b11111111111111111111111111111111}u`, rhs: `1u`, pass: false },
{ lhs: `${0b11111111111111111111111111111111}u`, rhs: `31u`, pass: false },
// Same cases should pass if lhs is signed
{ lhs: `${signed(0b11000000000000000000000000000000)}i`, rhs: `1u`, pass: true },
{ lhs: `${signed(0b11111111111111111111111111111111)}i`, rhs: `1u`, pass: true },
{ lhs: `${signed(0b11111111111111111111111111111111)}i`, rhs: `31u`, pass: true },

// Shift by negative is an error
{ lhs: `1`, rhs: `-1`, pass: false },
{ lhs: `1i`, rhs: `-1`, pass: false },
{ lhs: `1u`, rhs: `-1`, pass: false },

// Signed overflow (sign change) for abstract
{ lhs: `1`, rhs: `63`, pass: false },
{ lhs: `2`, rhs: `62`, pass: false },
{
  lhs: `${0b0100000000000000000000000000000000000000000000000000000000000000}`,
  rhs: `1u`,
  pass: false
}];


g.test('shift_left_concrete').
desc('Tests validation of binary left shift of concrete values').
params((u) =>
u.
combine('case', kLeftShiftCases) //
.combine('vectorize', [undefined, 2, 3, 4])
).
fn((t) => {
  const lhs = t.params.case.lhs;
  const rhs = t.params.case.rhs;
  const vec_size = t.params.vectorize;

  const code = `
@compute @workgroup_size(1)
fn main() {
    const r = ${vectorize(lhs, vec_size)} << ${vectorize(rhs, vec_size)};
}
    `;
  t.expectCompileResult(t.params.case.pass, code);
});

const kRightShiftCases = [
// rhs >= bitwidth fails
{ lhs: `0u`, rhs: `31u`, pass: true },
{ lhs: `0u`, rhs: `32u`, pass: false },
{ lhs: `0u`, rhs: `33u`, pass: false },
{ lhs: `0u`, rhs: `1000u`, pass: false },
{ lhs: `0u`, rhs: `0xFFFFFFFFu`, pass: false },

{ lhs: `0i`, rhs: `31u`, pass: true },
{ lhs: `0i`, rhs: `32u`, pass: false },
{ lhs: `0i`, rhs: `33u`, pass: false },
{ lhs: `0i`, rhs: `1000u`, pass: false },
{ lhs: `0i`, rhs: `0xFFFFFFFFu`, pass: false },

// Shift by negative is an error
{ lhs: `1`, rhs: `-1`, pass: false },
{ lhs: `1i`, rhs: `-1`, pass: false },
{ lhs: `1u`, rhs: `-1`, pass: false }];


g.test('shift_right_concrete').
desc('Tests validation of binary right shift of concrete values').
params((u) =>
u.
combine('case', kRightShiftCases) //
.combine('vectorize', [undefined, 2, 3, 4])
).
fn((t) => {
  const lhs = t.params.case.lhs;
  const rhs = t.params.case.rhs;
  const vec_size = t.params.vectorize;

  const code = `
@compute @workgroup_size(1)
fn main() {
    const r = ${vectorize(lhs, vec_size)} >> ${vectorize(rhs, vec_size)};
}
    `;
  t.expectCompileResult(t.params.case.pass, code);
});

g.test('shift_left_abstract').
desc('Validates that the result when the LHS is abstract is also abstract').
fn((t) => {
  const wgsl = `
    const lhs = 0xfffff0000; // too large for 32 bits
    const res = lhs << 4u;
    const_assert res == 0xfffff00000;`;
  t.expectCompileResult(true, wgsl);
});

g.test('shift_right_abstract').
desc('Validates that the result when the LHS is abstract is also abstract').
fn((t) => {
  const wgsl = `
    const lhs = 0xfffff0000; // too large for 32 bits
    const res = lhs >> 1u;
    const_assert res == 0x7ffff8000;`;
  t.expectCompileResult(true, wgsl);
});

g.test('partial_eval_errors').
desc('Tests partial evaluation errors for left and right shift').
params((u) =>
u.
combine('op', ['<<', '>>']).
combine('type', ['i32', 'u32']).
beginSubcases().
combine('stage', ['shader', 'pipeline']).
combine('value', [31, 32, 33, 64])
).
fn((t) => {
  const u32 = Type.u32;
  let rhs = 'o';
  if (t.params.stage === 'shader') {
    rhs = `${u32.create(t.params.value).wgsl()}`;
  }
  const wgsl = `
override o = 0u;
fn foo() -> ${t.params.type} {
  var v : ${t.params.type} = 0;
  return v ${t.params.op} ${rhs};
}`;

  const expect = t.params.value < 32;
  if (t.params.stage === 'shader') {
    t.expectCompileResult(expect, wgsl);
  } else {
    const constants = {};
    constants['o'] = t.params.value;
    t.expectPipelineResult({
      expectedResult: expect,
      code: wgsl,
      constants,
      reference: ['o', 'foo()']
    });
  }
});