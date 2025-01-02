/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for array indexing expressions
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import {
  False,
  True,
  Type,

  array,
  f32,
  scalarTypeOf } from
'../../../../../util/conversion.js';
import { align } from '../../../../../util/math.js';

import { allInputSources, basicExpressionBuilder, run } from '../../expression.js';

export const g = makeTestGroup(GPUTest);

g.test('concrete_scalar').
specURL('https://www.w3.org/TR/WGSL/#array-access-expr').
desc(`Test indexing of an array of concrete scalars`).
params((u) =>
u.
combine(
  'inputSource',
  // 'uniform' address space requires array stride to be multiple of 16 bytes
  allInputSources.filter((s) => s !== 'uniform')
).
combine('elementType', ['i32', 'u32', 'f32', 'f16']).
combine('indexType', ['i32', 'u32'])
).
beforeAllSubcases((t) => {
  if (t.params.elementType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.elementType];
  const indexType = Type[t.params.indexType];
  const cases = [
  {
    input: [
    array(
      /* 0 */elementType.create(10),
      /* 1 */elementType.create(11),
      /* 2 */elementType.create(12)
    ),
    indexType.create(0)],

    expected: elementType.create(10)
  },
  {
    input: [
    array(
      /* 0 */elementType.create(20),
      /* 1 */elementType.create(21),
      /* 2 */elementType.create(22)
    ),
    indexType.create(1)],

    expected: elementType.create(21)
  },
  {
    input: [
    array(
      /* 0 */elementType.create(30),
      /* 1 */elementType.create(31),
      /* 2 */elementType.create(32)
    ),
    indexType.create(2)],

    expected: elementType.create(32)
  }];

  await run(
    t,
    basicExpressionBuilder((ops) => `${ops[0]}[${ops[1]}]`),
    [Type.array(3, elementType), indexType],
    elementType,
    t.params,
    cases
  );
});

g.test('bool').
specURL('https://www.w3.org/TR/WGSL/#array-access-expr').
desc(`Test indexing of an array of booleans`).
params((u) =>
u.
combine(
  'inputSource',
  // 'uniform' address space requires array stride to be multiple of 16 bytes
  allInputSources.filter((s) => s !== 'uniform')
).
combine('indexType', ['i32', 'u32'])
).
fn(async (t) => {
  const indexType = Type[t.params.indexType];
  const cases = [
  {
    input: [array(True, False, True), indexType.create(0)],
    expected: True
  },
  {
    input: [array(True, False, True), indexType.create(1)],
    expected: False
  },
  {
    input: [array(True, False, True), indexType.create(2)],
    expected: True
  }];

  await run(
    t,
    basicExpressionBuilder((ops) => `${ops[0]}[${ops[1]}]`),
    [Type.array(3, Type.bool), indexType],
    Type.bool,
    t.params,
    cases
  );
});

g.test('abstract_scalar').
specURL('https://www.w3.org/TR/WGSL/#array-access-expr').
desc(`Test indexing of an array of scalars`).
params((u) =>
u.
combine('elementType', ['abstract-int', 'abstract-float']).
combine('indexType', ['i32', 'u32'])
).
fn(async (t) => {
  const elementType = Type[t.params.elementType];
  const indexType = Type[t.params.indexType];
  const cases = [
  {
    input: [
    array(
      /* 0 */elementType.create(0x10_00000000),
      /* 1 */elementType.create(0x11_00000000),
      /* 2 */elementType.create(0x12_00000000)
    ),
    indexType.create(0)],

    expected: f32(0x10)
  },
  {
    input: [
    array(
      /* 0 */elementType.create(0x20_00000000),
      /* 1 */elementType.create(0x21_00000000),
      /* 2 */elementType.create(0x22_00000000)
    ),
    indexType.create(1)],

    expected: f32(0x21)
  },
  {
    input: [
    array(
      /* 0 */elementType.create(0x30_00000000),
      /* 1 */elementType.create(0x31_00000000),
      /* 2 */elementType.create(0x32_00000000)
    ),
    indexType.create(2)],

    expected: f32(0x32)
  }];

  await run(
    t,
    basicExpressionBuilder((ops) => `${ops[0]}[${ops[1]}] / 0x100000000`),
    [Type.array(3, elementType), indexType],
    Type.f32,
    { inputSource: 'const' },
    cases
  );
});

g.test('runtime_sized').
specURL('https://www.w3.org/TR/WGSL/#array-access-expr').
desc(`Test indexing of a runtime sized array`).
params((u) =>
u.
combine('elementType', [
'i32',
'u32',
'f32',
'f16',
'vec4i',
'vec2u',
'vec3f',
'vec2h']
).
combine('indexType', ['i32', 'u32'])
).
beforeAllSubcases((t) => {
  if (scalarTypeOf(Type[t.params.elementType]).kind === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const elementType = Type[t.params.elementType];
  const valueArrayType = Type.array(0, elementType);
  const indexType = Type[t.params.indexType];
  const indexArrayType = Type.array(0, indexType);

  const wgsl = `
${scalarTypeOf(elementType).kind === 'f16' ? 'enable f16;' : ''}

@group(0) @binding(0) var<storage, read> input_values : ${valueArrayType};
@group(0) @binding(1) var<storage, read> input_indices : ${indexArrayType};
@group(0) @binding(2) var<storage, read_write> output : ${valueArrayType};

@compute @workgroup_size(16)
fn main(@builtin(local_invocation_index) invocation_id : u32) {
  let index = input_indices[invocation_id];
  output[invocation_id] = input_values[index];
}
`;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  const values = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53];
  const indices = [9, 0, 14, 10, 12, 4, 15, 3, 5, 6, 11, 2, 8, 13, 7, 1];

  const inputValues = values.map((i) => elementType.create(i));
  const inputIndices = indices.map((i) => indexType.create(i));
  const expected = indices.map((i) => inputValues[i]);

  const bufferSize = (arr) => {
    let offset = 0;
    let alignment = 0;
    for (const value of arr) {
      alignment = Math.max(alignment, value.type.alignment);
      offset = align(offset, value.type.alignment) + value.type.size;
    }
    return align(offset, alignment);
  };

  const toArray = (arr) => {
    const array = new Uint8Array(bufferSize(arr));
    let offset = 0;
    for (const value of arr) {
      offset = align(offset, value.type.alignment);
      value.copyTo(array, offset);
      offset += value.type.size;
    }
    return array;
  };

  const inputArrayBuffer = t.makeBufferWithContents(toArray(inputValues), GPUBufferUsage.STORAGE);
  const inputIndexBuffer = t.makeBufferWithContents(
    toArray(inputIndices),
    GPUBufferUsage.STORAGE
  );
  const outputBuffer = t.createBufferTracked({
    size: bufferSize(expected),
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: { buffer: inputArrayBuffer } },
    { binding: 1, resource: { buffer: inputIndexBuffer } },
    { binding: 2, resource: { buffer: outputBuffer } }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(outputBuffer, toArray(expected));
});

g.test('vector').
specURL('https://www.w3.org/TR/WGSL/#array-access-expr').
desc(`Test indexing of an array of vectors`).
params((u) =>
u.
combine('inputSource', allInputSources).
expand('elementType', (t) =>
t.inputSource === 'uniform' ?
['vec4i', 'vec4u', 'vec4f'] :
['vec4i', 'vec4u', 'vec4f', 'vec4h']
).
combine('indexType', ['i32', 'u32'])
).
beforeAllSubcases((t) => {
  if (t.params.elementType === 'vec4h') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.elementType];
  const indexType = Type[t.params.indexType];
  const cases = [
  {
    input: [
    array(
      /* 0 */elementType.create([0x10, 0x11, 0x12, 0x13]),
      /* 1 */elementType.create([0x14, 0x15, 0x16, 0x17]),
      /* 2 */elementType.create([0x18, 0x19, 0x1a, 0x1b])
    ),
    indexType.create(0)],

    expected: elementType.create([0x10, 0x11, 0x12, 0x13])
  },
  {
    input: [
    array(
      /* 0 */elementType.create([0x20, 0x21, 0x22, 0x23]),
      /* 1 */elementType.create([0x24, 0x25, 0x26, 0x27]),
      /* 2 */elementType.create([0x28, 0x29, 0x2a, 0x2b])
    ),
    indexType.create(1)],

    expected: elementType.create([0x24, 0x25, 0x26, 0x27])
  },
  {
    input: [
    array(
      /* 0 */elementType.create([0x30, 0x31, 0x32, 0x33]),
      /* 1 */elementType.create([0x34, 0x35, 0x36, 0x37]),
      /* 2 */elementType.create([0x38, 0x39, 0x3a, 0x3b])
    ),
    indexType.create(2)],

    expected: elementType.create([0x38, 0x39, 0x3a, 0x3b])
  }];

  await run(
    t,
    basicExpressionBuilder((ops) => `${ops[0]}[${ops[1]}]`),
    [Type.array(3, elementType), indexType],
    elementType,
    t.params,
    cases
  );
});

g.test('matrix').
specURL('https://www.w3.org/TR/WGSL/#array-access-expr').
desc(`Test indexing of an array of matrices`).
params((u) =>
u.
combine('inputSource', allInputSources).
combine('elementType', ['f16', 'f32']).
beginSubcases().
combine('columns', [2, 3, 4]).
combine('rows', [2, 3, 4]).
combine('indexType', ['i32', 'u32']).
filter((u) => {
  if (u.inputSource !== 'uniform') {
    return true;
  }
  const mat = Type.mat(u.columns, u.rows, Type[u.elementType]);
  return (align(mat.size, mat.alignment) & 15) === 0;
})
).
beforeAllSubcases((t) => {
  if (t.params.elementType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const elementType = Type[t.params.elementType];
  const indexType = Type[t.params.indexType];
  const matrixType = Type.mat(t.params.columns, t.params.rows, elementType);
  const buildMat = (index) => {
    const elements = [];
    for (let c = 0; c < t.params.rows; c++) {
      for (let r = 0; r < t.params.columns; r++) {
        elements.push(index * 100 + c * 10 + r);
      }
    }
    return matrixType.create(elements);
  };
  const matrices = [buildMat(0), buildMat(1), buildMat(2)];
  await run(
    t,
    basicExpressionBuilder((ops) => `${ops[0]}[${ops[1]}]`),
    [Type.array(3, matrixType), indexType],
    matrixType,
    t.params,
    [
    {
      input: [array(...matrices), indexType.create(1)],
      expected: matrices[1]
    }]

  );
});