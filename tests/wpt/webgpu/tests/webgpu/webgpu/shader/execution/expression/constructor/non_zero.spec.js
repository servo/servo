/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for value constructors from components
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
import {
  ArrayValue,


  Type,



  scalarTypeOf,
  vec2,
  vec3 } from
'../../../../util/conversion.js';
import { FP } from '../../../../util/floating_point.js';
import {

  allInputSources,
  basicExpressionBuilder,
  run } from
'../expression.js';

export const g = makeTestGroup(GPUTest);

/** @returns true if 'v' is 'min' or 'max' */
function isMinOrMax(v) {
  return v === 'min' || v === 'max';
}

/** A list of concrete types to test for the given abstract-numeric type */
const kConcreteTypesForAbstractType = {
  'abstract-float': ['f32', 'f16'],
  'abstract-int': ['f32', 'f16', 'i32', 'u32'],
  'vec3<abstract-int>': ['vec3f', 'vec3h', 'vec3i', 'vec3u'],
  'vec4<abstract-float>': ['vec4f', 'vec4h'],
  'mat2x3<abstract-float>': ['mat2x3f', 'mat2x3h']
};

/**
 * @returns the lowest finite value for 'kind' if 'v' is 'min',
 *          the highest finite value for 'kind' if 'v' is 'max',
 *          otherwise returns 'v'
 */
function valueFor(v, kind) {
  if (!isMinOrMax(v)) {
    return v;
  }
  switch (kind) {
    case 'bool':
      return v === 'min' ? 0 : 1;
    case 'i32':
      return v === 'min' ? -0x80000000 : 0x7fffffff;
    case 'u32':
      return v === 'min' ? 0 : 0xffffffff;
    case 'f32':
      return v === 'min' ? FP['f32'].constants().negative.min : FP['f32'].constants().positive.max;
    case 'f16':
      return v === 'min' ? FP['f16'].constants().negative.min : FP['f16'].constants().positive.max;
  }
}

g.test('scalar_identity').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a scalar constructed from a value of the same type produces the expected value`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('type', ['bool', 'i32', 'u32', 'f32', 'f16']).
combine('value', ['min', 'max', 1, 2, 5, 100])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
  t.skipIf(t.params.type === 'bool' && !isMinOrMax(t.params.value));
}).
fn(async (t) => {
  const type = Type[t.params.type];
  const value = valueFor(t.params.value, t.params.type);
  await run(
    t,
    basicExpressionBuilder((ops) => `${type}(${ops[0]})`),
    [type],
    type,
    t.params,
    [{ input: [type.create(value)], expected: type.create(value) }]
  );
});

g.test('vector_identity').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a vector constructed from a value of the same type produces the expected value`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('type', ['bool', 'i32', 'u32', 'f32', 'f16']).
combine('width', [2, 3, 4]).
combine('infer_type', [false, true])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.type];
  const vectorType = Type.vec(t.params.width, elementType);
  const elements = [];
  const fn = t.params.infer_type ? `vec${t.params.width}` : `${vectorType}`;
  for (let i = 0; i < t.params.width; i++) {
    if (t.params.type === 'bool') {
      elements.push(i & 1);
    } else {
      elements.push((i + 1) * 10);
    }
  }

  await run(
    t,
    basicExpressionBuilder((ops) => `${fn}(${ops[0]})`),
    [vectorType],
    vectorType,
    t.params,
    [
    {
      input: vectorType.create(elements),
      expected: vectorType.create(elements)
    }]

  );
});

g.test('concrete_vector_splat').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a vector constructed from a single concrete scalar produces the expected value`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('type', ['bool', 'i32', 'u32', 'f32', 'f16']).
combine('value', ['min', 'max', 1, 2, 5, 100]).
combine('width', [2, 3, 4]).
combine('infer_type', [false, true])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
  t.skipIf(t.params.type === 'bool' && !isMinOrMax(t.params.value));
}).
fn(async (t) => {
  const value = valueFor(t.params.value, t.params.type);
  const elementType = Type[t.params.type];
  const vectorType = Type.vec(t.params.width, elementType);
  const fn = t.params.infer_type ? `vec${t.params.width}` : `${vectorType}`;
  await run(
    t,
    basicExpressionBuilder((ops) => `${fn}(${ops[0]})`),
    [elementType],
    vectorType,
    t.params,
    [{ input: [elementType.create(value)], expected: vectorType.create(value) }]
  );
});

g.test('abstract_vector_splat').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a vector constructed from a single abstract scalar produces the expected value`).
params((u) =>
u.
combine('abstract_type', ['abstract-int', 'abstract-float']).
expand('concrete_type', (t) => kConcreteTypesForAbstractType[t.abstract_type]).
combine('value', [1, 2, 5, 100]).
combine('width', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.concrete_type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const suffix = t.params.abstract_type === 'abstract-float' ? '.0' : '';
  const concreteElementType = Type[t.params.concrete_type];
  const concreteVectorType = Type.vec(t.params.width, concreteElementType);
  const fn = `vec${t.params.width}`;
  await run(
    t,
    basicExpressionBuilder((_) => `${fn}(${t.params.value * 0x100000000}${suffix}) / 0x100000000`),
    [],
    concreteVectorType,
    { inputSource: 'const', constEvaluationMode: 'direct' },
    [{ input: [], expected: concreteVectorType.create(t.params.value) }]
  );
});

g.test('concrete_vector_elements').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a vector constructed from concrete element values produces the expected value`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('type', ['bool', 'i32', 'u32', 'f32', 'f16']).
combine('width', [2, 3, 4]).
combine('infer_type', [false, true])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.type];
  const vectorType = Type.vec(t.params.width, elementType);
  const elements = [];
  const fn = t.params.infer_type ? `vec${t.params.width}` : `${vectorType}`;
  for (let i = 0; i < t.params.width; i++) {
    if (t.params.type === 'bool') {
      elements.push(i & 1);
    } else {
      elements.push((i + 1) * 10);
    }
  }

  await run(
    t,
    basicExpressionBuilder((ops) => `${fn}(${ops.join(', ')})`),
    elements.map((e) => elementType),
    vectorType,
    t.params,
    [
    {
      input: elements.map((v) => elementType.create(v)),
      expected: vectorType.create(elements)
    }]

  );
});

g.test('abstract_vector_elements').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a vector constructed from abstract element values produces the expected value`).
params((u) =>
u.
combine('abstract_type', ['abstract-int', 'abstract-float']).
expand('concrete_type', (t) => kConcreteTypesForAbstractType[t.abstract_type]).
combine('width', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.concrete_type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const suffix = t.params.abstract_type === 'abstract-float' ? '.0' : '';
  const concreteElementType = Type[t.params.concrete_type];
  const concreteVectorType = Type.vec(t.params.width, concreteElementType);
  const fn = `vec${t.params.width}`;
  const elements = [];
  for (let i = 0; i < t.params.width; i++) {
    elements.push((i + 1) * 10);
  }
  await run(
    t,
    basicExpressionBuilder(
      (_) => `${fn}(${elements.map((v) => `${v * 0x100000000}${suffix}`).join(', ')}) / 0x100000000`
    ),
    [],
    concreteVectorType,
    { inputSource: 'const', constEvaluationMode: 'direct' },
    [{ input: [], expected: concreteVectorType.create(elements) }]
  );
});

const kMixSignatures = [
'2s', //   [vec2,   scalar]
's2', //   [scalar, vec2]
'2ss', //  [vec2,   scalar,   scalar]
's2s', //  [scalar, vec2,     scalar]
'ss2', //  [scalar, scalar,   vec2  ]
'22', //   [vec2,   vec2]
'3s', //   [vec3,   scalar]
's3' //   [scalar, vec3]
];

g.test('concrete_vector_mix').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `Test that a vector constructed from a mix of concrete element values and sub-vectors produces the expected value`
).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('type', ['bool', 'i32', 'u32', 'f32', 'f16']).
combine('signature', kMixSignatures).
combine('infer_type', [false, true])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.type];
  let width = 0;
  const elementValue = (i) => t.params.type === 'bool' ? i & 1 : (i + 1) * 10;
  const elements = [];
  const nextValue = () => {
    const value = elementValue(width++);
    elements.push(value);
    return elementType.create(value);
  };
  const args = [];
  for (const c of t.params.signature) {
    switch (c) {
      case '2':
        args.push(vec2(nextValue(), nextValue()));
        break;
      case '3':
        args.push(vec3(nextValue(), nextValue(), nextValue()));
        break;
      case 's':
        args.push(nextValue());
        break;
    }
  }
  const vectorType = Type.vec(width, elementType);
  const fn = t.params.infer_type ? `vec${width}` : `${vectorType}`;
  await run(
    t,
    basicExpressionBuilder((ops) => `${fn}(${ops.join(', ')})`),
    args.map((e) => e.type),
    vectorType,
    t.params,
    [
    {
      input: args,
      expected: vectorType.create(elements)
    }]

  );
});

g.test('abstract_vector_mix').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(
  `Test that a vector constructed from a mix of abstract element values and sub-vectors produces the expected value`
).
params((u) =>
u.
combine('abstract_type', ['abstract-int', 'abstract-float']).
expand('concrete_type', (t) => kConcreteTypesForAbstractType[t.abstract_type]).
combine('signature', kMixSignatures)
).
beforeAllSubcases((t) => {
  if (t.params.concrete_type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  let width = 0;
  const suffix = t.params.abstract_type === 'abstract-float' ? '.0' : '';
  const concreteElementType = Type[t.params.concrete_type];
  const elementValue = (i) => (i + 1) * 10;
  const elements = [];
  const nextValue = () => {
    const value = elementValue(width++);
    elements.push(value);
    return `${value * 0x100000000}${suffix}`;
  };
  const args = [];
  for (const c of t.params.signature) {
    switch (c) {
      case '2':
        args.push(`vec2(${nextValue()}, ${nextValue()})`);
        break;
      case '3':
        args.push(`vec3(${nextValue()}, ${nextValue()}, ${nextValue()})`);
        break;
      case 's':
        args.push(`${nextValue()}`);
        break;
    }
  }
  const concreteVectorType = Type.vec(width, concreteElementType);
  const fn = `vec${width}`;
  await run(
    t,
    basicExpressionBuilder((_) => `${fn}(${args.join(', ')}) / 0x100000000`),
    [],
    concreteVectorType,
    { inputSource: 'const', constEvaluationMode: 'direct' },
    [
    {
      input: [],
      expected: concreteVectorType.create(elements)
    }]

  );
});

g.test('matrix_identity').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a matrix constructed from a value of the same type produces the expected value`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('type', ['f32', 'f16']).
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4]).
combine('infer_type', [false, true])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.type];
  const matrixType = Type.mat(t.params.columns, t.params.rows, elementType);
  const elements = [];
  for (let column = 0; column < t.params.columns; column++) {
    for (let row = 0; row < t.params.rows; row++) {
      elements.push((column + 1) * 10 + (row + 1));
    }
  }
  const fn = t.params.infer_type ? `mat${t.params.columns}x${t.params.rows}` : `${matrixType}`;
  await run(
    t,
    basicExpressionBuilder((ops) => `${fn}(${ops[0]})`),
    [matrixType],
    matrixType,
    t.params,
    [
    {
      input: matrixType.create(elements),
      expected: matrixType.create(elements)
    }]

  );
});

g.test('concrete_matrix_elements').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a matrix constructed from concrete element values produces the expected value`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('type', ['f32', 'f16']).
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4]).
combine('infer_type', [false, true])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.type];
  const matrixType = Type.mat(t.params.columns, t.params.rows, elementType);
  const elements = [];
  for (let column = 0; column < t.params.columns; column++) {
    for (let row = 0; row < t.params.rows; row++) {
      elements.push((column + 1) * 10 + (row + 1));
    }
  }
  const fn = t.params.infer_type ? `mat${t.params.columns}x${t.params.rows}` : `${matrixType}`;
  await run(
    t,
    basicExpressionBuilder((ops) => `${fn}(${ops.join(', ')})`),
    elements.map((e) => elementType),
    matrixType,
    t.params,
    [
    {
      input: elements.map((e) => elementType.create(e)),
      expected: matrixType.create(elements)
    }]

  );
});

g.test('abstract_matrix_elements').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a matrix constructed from concrete element values produces the expected value`).
params((u) =>
u.
combine('concrete_type', ['f32', 'f16']).
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.concrete_type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const concreteElementType = Type[t.params.concrete_type];
  const concreteMatrixType = Type.mat(t.params.columns, t.params.rows, concreteElementType);
  const elements = [];
  for (let column = 0; column < t.params.columns; column++) {
    for (let row = 0; row < t.params.rows; row++) {
      elements.push((column + 1) * 10 + (row + 1));
    }
  }
  const fn = `mat${t.params.columns}x${t.params.rows}`;
  await run(
    t,
    basicExpressionBuilder(
      (_) => `${fn}(${elements.map((v) => `${v * 0x100000000}.0`).join(', ')}) * (1.0 / 0x100000000)`
    ),
    [],
    concreteMatrixType,
    { inputSource: 'const', constEvaluationMode: 'direct' },
    [
    {
      input: [],
      expected: concreteMatrixType.create(elements)
    }]

  );
});

g.test('concrete_matrix_column_vectors').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a matrix constructed from concrete column vectors produces the expected value`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('type', ['f32', 'f16']).
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4]).
combine('infer_type', [false, true])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.type];
  const columnType = Type.vec(t.params.rows, elementType);
  const matrixType = Type.mat(t.params.columns, t.params.rows, elementType);
  const elements = [];
  const columnVectors = [];
  for (let column = 0; column < t.params.columns; column++) {
    const columnElements = [];
    for (let row = 0; row < t.params.rows; row++) {
      const v = (column + 1) * 10 + (row + 1);
      elements.push(v);
      columnElements.push(v);
    }
    columnVectors.push(columnType.create(columnElements));
  }
  const fn = t.params.infer_type ? `mat${t.params.columns}x${t.params.rows}` : `${matrixType}`;
  await run(
    t,
    basicExpressionBuilder((ops) => `${fn}(${ops.join(', ')})`),
    columnVectors.map((v) => v.type),
    matrixType,
    t.params,
    [
    {
      input: columnVectors,
      expected: matrixType.create(elements)
    }]

  );
});

g.test('abstract_matrix_column_vectors').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that a matrix constructed from abstract column vectors produces the expected value`).
params((u) =>
u.
combine('concrete_type', ['f32', 'f16']).
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4])
).
beforeAllSubcases((t) => {
  if (t.params.concrete_type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const concreteElementType = Type[t.params.concrete_type];
  const concreteMatrixType = Type.mat(t.params.columns, t.params.rows, concreteElementType);
  const elements = [];
  const columnVectors = [];
  for (let column = 0; column < t.params.columns; column++) {
    const columnElements = [];
    for (let row = 0; row < t.params.rows; row++) {
      const v = (column + 1) * 10 + (row + 1);
      elements.push(v);
      columnElements.push(`${v * 0x100000000}`);
    }
    columnVectors.push(`vec${t.params.rows}(${columnElements.join(', ')})`);
  }
  const fn = `mat${t.params.columns}x${t.params.rows}`;
  await run(
    t,
    basicExpressionBuilder((_) => `${fn}(${columnVectors.join(', ')}) * (1.0 / 0x100000000)`),
    [],
    concreteMatrixType,
    { inputSource: 'const', constEvaluationMode: 'direct' },
    [
    {
      input: [],
      expected: concreteMatrixType.create(elements)
    }]

  );
});

g.test('concrete_array_elements').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that an array constructed from concrete element values produces the expected value`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('type', ['bool', 'i32', 'u32', 'f32', 'f16', 'vec3f', 'vec4i']).
combine('length', [1, 5, 10]).
combine('infer_type', [false, true])
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.type];
  const arrayType = Type.array(t.params.length, elementType);
  const elements = [];
  for (let i = 0; i < t.params.length; i++) {
    elements.push((i + 1) * 10);
  }
  const fn = t.params.infer_type ? `array` : `${arrayType}`;
  await run(
    t,
    basicExpressionBuilder((ops) => `${fn}(${ops.join(', ')})`),
    elements.map((e) => elementType),
    arrayType,
    t.params,
    [
    {
      input: elements.map((e) => elementType.create(e)),
      expected: arrayType.create(elements)
    }]

  );
});

g.test('abstract_array_elements').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that an array constructed from element values produces the expected value`).
params((u) =>
u.
combine('abstract_type', [
'abstract-int',
'abstract-float',
'vec3<abstract-int>',
'vec4<abstract-float>',
'mat2x3<abstract-float>']
).
expand('concrete_type', (t) => kConcreteTypesForAbstractType[t.abstract_type]).
combine('length', [1, 5, 10])
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(Type[t.params.concrete_type]).kind === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const count = t.params.length;
  const concreteElementType = Type[t.params.concrete_type];
  const concreteArrayType = Type.array(count, concreteElementType);
  const elements = [];
  let i = 0;
  const nextValue = () => ++i * 10;
  for (let i = 0; i < count; i++) {
    switch (t.params.abstract_type) {
      case 'abstract-int':{
          const value = nextValue();
          elements.push({ args: `${value}`, value: concreteElementType.create(value) });
          break;
        }
      case 'abstract-float':{
          const value = nextValue();
          elements.push({ args: `${value}.0`, value: concreteElementType.create(value) });
          break;
        }
      case 'vec3<abstract-int>':{
          const x = nextValue();
          const y = nextValue();
          const z = nextValue();
          elements.push({
            args: `vec3(${x}, ${y}, ${z})`,
            value: concreteElementType.create([x, y, z])
          });
          break;
        }
      case 'vec4<abstract-float>':{
          const x = nextValue();
          const y = nextValue();
          const z = nextValue();
          const w = nextValue();
          elements.push({
            args: `vec4(${x}.0, ${y}.0, ${z}.0, ${w}.0)`,
            value: concreteElementType.create([x, y, z, w])
          });
          break;
        }
      case 'mat2x3<abstract-float>':{
          const e00 = nextValue();
          const e01 = nextValue();
          const e02 = nextValue();
          const e10 = nextValue();
          const e11 = nextValue();
          const e12 = nextValue();
          elements.push({
            args: `mat2x3(vec3(${e00}.0, ${e01}.0, ${e02}.0), vec3(${e10}.0, ${e11}.0, ${e12}.0))`,
            value: concreteElementType.create([e00, e01, e02, e10, e11, e12])
          });
          break;
        }
    }
  }
  const fn = `array`;
  await run(
    t,
    basicExpressionBuilder((_) => `${fn}(${elements.map((e) => e.args).join(', ')})`),
    [],
    concreteArrayType,
    { inputSource: 'const', constEvaluationMode: 'direct' },
    [
    {
      input: [],
      expected: new ArrayValue(elements.map((e) => e.value))
    }]

  );
});

g.test('structure').
specURL('https://www.w3.org/TR/WGSL/#value-constructor-builtin-function').
desc(`Test that an structure constructed from element values produces the expected value`).
params((u) =>
u.
combine('member_types', [
['bool'],
['u32'],
['vec3f'],
['i32', 'u32'],
['i32', 'f16', 'vec4i', 'mat3x2f'],
['bool', 'u32', 'f16', 'vec3f', 'vec2i'],
['i32', 'u32', 'f32', 'f16', 'vec3f', 'vec4i']]
).
combine('nested', [false, true]).
beginSubcases().
expand('member_index', (t) => t.member_types.map((_, i) => i))
).
beforeAllSubcases((t) => {
  if (t.params.member_types.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const memberType = Type[t.params.member_types[t.params.member_index]];
  const values = t.params.member_types.map((ty, i) => Type[ty].create(i));

  const builder = basicExpressionBuilder((ops) =>
  t.params.nested ?
  `OuterStruct(10, MyStruct(${ops.join(', ')}), 20).inner.member_${t.params.member_index}` :
  `MyStruct(${ops.join(', ')}).member_${t.params.member_index}`
  );
  await run(
    t,
    (params) => {
      return `
${t.params.member_types.includes('f16') ? 'enable f16;' : ''}

${builder(params)}

struct MyStruct {
${t.params.member_types.map((ty, i) => `  member_${i} : ${ty},`).join('\n')}
};
struct OuterStruct {
  pre : i32,
  inner : MyStruct,
  post : i32,
};
`;
    },
    t.params.member_types.map((ty) => Type[ty]),
    memberType,
    { inputSource: 'const' },
    [{ input: values, expected: values[t.params.member_index] }]
  );
});