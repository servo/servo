/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests to check array clamping in shaders is correctly implemented including vector / matrix indexing
`;
import { params, poptions } from '../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert } from '../../../common/framework/util/util.js';
import { GPUTest } from '../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

// Utilities that should probably live in some shared place.
function copyArrayBuffer(src) {
  const dst = new ArrayBuffer(src.byteLength);
  new Uint8Array(dst).set(new Uint8Array(src));
  return dst;
}

const kUintMax = 4294967295;
const kIntMax = 2147483647;

// A small utility to test shaders:
//  - it wraps the source into a small harness that checks the runTest() function returns 0.
//  - it runs the shader with the testBindings set as bindgroup 0.
//
// The shader also has access to a uniform value that's equal to 1u to avoid constant propagation
// in the shader compiler.
function runShaderTest(t, stage, testSource, testBindings) {
  assert(stage === GPUShaderStage.COMPUTE, 'Only know how to deal with compute for now');

  const [constantsBuffer, constantsInit] = t.device.createBufferMapped({
    size: 4,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.UNIFORM,
  });

  const constantsData = new Uint32Array(constantsInit);
  constantsData[0] = 1;
  constantsBuffer.unmap();

  const resultBuffer = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE,
  });

  const source = `#version 450
    layout(std140, set = 1, binding = 0) uniform Constants {
      uint one;
    };
    layout(std430, set = 1, binding = 1) buffer Result {
      uint result;
    };

    ${testSource}

    void main() {
      result = runTest();
    }`;

  const pipeline = t.device.createComputePipeline({
    computeStage: {
      entryPoint: 'main',
      module: t.makeShaderModule('compute', { glsl: source }),
    },
  });

  const group = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(1),
    entries: [
      { binding: 0, resource: { buffer: constantsBuffer } },
      { binding: 1, resource: { buffer: resultBuffer } },
    ],
  });

  const testGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: testBindings,
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, testGroup);
  pass.setBindGroup(1, group);
  pass.dispatch(1);
  pass.endPass();

  t.queue.submit([encoder.finish()]);

  t.expectContents(resultBuffer, new Uint32Array([0]));
}

// The definition of base types for aggregate types, for example float, uint, etc.

const baseTypes = {
  // TODO bools
  uint: {
    name: 'uint',
    byteSize: 4,
    glslPrefix: 'u',
    glslZero: '0u',
    fillBuffer(data, zeroStart, size) {
      const typedData = new Uint32Array(data);
      typedData.fill(42);
      for (let i = 0; i < size / 4; i++) {
        typedData[zeroStart / 4 + i] = 0;
      }
    },
  },

  int: {
    name: 'int',
    byteSize: 4,
    glslPrefix: 'i',
    glslZero: '0',
    fillBuffer(data, zeroStart, size) {
      const typedData = new Int32Array(data);
      typedData.fill(42);
      for (let i = 0; i < size / 4; i++) {
        typedData[zeroStart / 4 + i] = 0;
      }
    },
  },

  float: {
    name: 'float',
    byteSize: 4,
    glslPrefix: '',
    glslZero: '0.0f',
    fillBuffer(data, zeroStart, size) {
      const typedData = new Float32Array(data);
      typedData.fill(42);
      for (let i = 0; i < size / 4; i++) {
        typedData[zeroStart / 4 + i] = 0;
      }
    },
  },

  bool: {
    name: 'bool',
    byteSize: 4,
    glslPrefix: 'b',
    glslZero: 'false',
    fillBuffer(data, zeroStart, size) {
      const typedData = new Uint32Array(data);
      typedData.fill(42);
      for (let i = 0; i < size / 4; i++) {
        typedData[zeroStart / 4 + i] = 0;
      }
    },
  },
};

// The definition of aggregate types.

const typeParams = (() => {
  const types = {};
  for (const baseTypeName of Object.keys(baseTypes)) {
    const baseType = baseTypes[baseTypeName];

    // Arrays
    types[`${baseTypeName}_sizedArray`] = {
      declaration: `${baseTypeName} data[3]`,
      length: 3,
      std140Length: 2 * 4 + 1,
      std430Length: 3,
      zero: baseType.glslZero,
      baseType,
    };

    types[`${baseTypeName}_unsizedArray`] = {
      declaration: `${baseTypeName} data[]`,
      length: 3,
      std140Length: 0, // Unused
      std430Length: 3,
      zero: baseType.glslZero,
      baseType,
      isUnsizedArray: true,
    };

    // Vectors
    for (let dimension = 2; dimension <= 4; dimension++) {
      types[`${baseTypeName}_vector${dimension}`] = {
        declaration: `${baseType.glslPrefix}vec${dimension} data`,
        length: dimension,
        std140Length: dimension,
        std430Length: dimension,
        zero: baseType.glslZero,
        baseType,
      };
    }
  }

  // Matrices, there are only float matrics in GLSL.
  for (const transposed of [false, true]) {
    for (let numColumns = 2; numColumns <= 4; numColumns++) {
      for (let numRows = 2; numRows <= 4; numRows++) {
        const majorDim = transposed ? numRows : numColumns;
        const minorDim = transposed ? numColumns : numRows;

        const std140SizePerMinorDim = 4;
        const std430SizePerMinorDim = minorDim === 3 ? 4 : minorDim;

        let typeName = `mat${numColumns}`;
        if (numColumns !== numRows) {
          typeName += `x${numRows}`;
        }

        types[(transposed ? 'transposed_' : '') + typeName] = {
          declaration: (transposed ? 'layout(row_major) ' : '') + `${typeName} data`,
          length: numColumns,
          std140Length: std140SizePerMinorDim * (majorDim - 1) + minorDim,
          std430Length: std430SizePerMinorDim * (majorDim - 1) + minorDim,
          zero: `vec${numRows}(0.0f)`,
          baseType: baseTypes['float'],
        };
      }
    }
  }

  return types;
})();

g.test('bufferMemory')
  .params(
    params()
      .combine(poptions('type', Object.keys(typeParams)))
      .combine([
        { memory: 'storage', access: 'read' },
        { memory: 'storage', access: 'write' },
        { memory: 'storage', access: 'atomic' },
        { memory: 'uniform', access: 'read' },
      ])

      // Unsized arrays are only supported with SSBOs
      .unless(p => typeParams[p.type].isUnsizedArray === true && p.memory !== 'storage')
      // Atomics are only supported with integers
      .unless(p => p.access === 'atomic' && !(typeParams[p.type].baseType.name in ['uint', 'int']))
  )
  .fn(async t => {
    const type = typeParams[t.params.type];
    const baseType = type.baseType;

    const indicesToTest = [
      // Write to the inside of the type so we can check the size computations were correct.
      '0',
      `${type.length} - 1`,

      // Check exact bounds
      '-1',
      `${type.length}`,

      // Check large offset
      '-1000000',
      '1000000',

      // Check with max uint
      `${kUintMax}`,
      `-1 * ${kUintMax}`,

      // Check with max int
      `${kIntMax}`,
      `-1 * ${kIntMax}`,
    ];

    let testSource = '';
    let byteSize = 0;

    // Declare the data that will be accessed to check robust access.
    if (t.params.memory === 'uniform') {
      testSource += `
        layout(std140, set = 0, binding = 0) uniform TestData {
          ${type.declaration};
        };`;
      byteSize = baseType.byteSize * type.std140Length;
    } else {
      testSource += `
        layout(std430, set = 0, binding = 0) buffer TestData {
          ${type.declaration};
        };`;
      byteSize = baseType.byteSize * type.std430Length;
    }

    // Build the test function that will do the tests.
    testSource += `
    uint runTest() {
  `;

    for (const indexToTest of indicesToTest) {
      // TODO check with constants too.
      const index = `(${indexToTest}) * one`;

      if (t.params.access === 'read') {
        testSource += `
          if(data[${index}] != ${type.zero}) {
            return __LINE__;
          }`;
      } else if (t.params.access === 'write') {
        testSource += `data[${index}] = ${type.zero};`;
      } else {
        testSource += `atomicAdd(data[${index}], 1);`;
      }
    }

    testSource += `
      return 0;
    }`;

    // Create a buffer that contains zeroes in the allowed access area, and 42s everywhere else.
    const [testBuffer, testInit] = t.device.createBufferMapped({
      size: 512,
      usage:
        GPUBufferUsage.COPY_SRC |
        GPUBufferUsage.UNIFORM |
        GPUBufferUsage.STORAGE |
        GPUBufferUsage.COPY_DST,
    });

    baseType.fillBuffer(testInit, 256, byteSize);
    const testInitCopy = copyArrayBuffer(testInit);
    testBuffer.unmap();

    // Run the shader, accessing the buffer.
    runShaderTest(t, GPUShaderStage.COMPUTE, testSource, [
      { binding: 0, resource: { buffer: testBuffer, offset: 256, size: byteSize } },
    ]);

    // Check that content of the buffer outside of the allowed area didn't change.
    t.expectSubContents(testBuffer, 0, new Uint8Array(testInitCopy.slice(0, 256)));
    const dataEnd = 256 + byteSize;
    t.expectSubContents(testBuffer, dataEnd, new Uint8Array(testInitCopy.slice(dataEnd, 512)));
  });

// TODO: also check other shader stages.
// TODO: also check global, function local, and shared variables.
// TODO: also check interface variables.
// TODO: also check storage texture access.
