/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests to check datatype clamping in shaders is correctly implemented for all indexable types
(vectors, matrices, sized/unsized arrays) visible to shaders in various ways.

TODO: add tests to check that textureLoad operations stay in-bounds.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert } from '../../../common/util/util.js';
import { Float16Array } from '../../../external/petamoriken/float16/float16.js';
import { GPUTest } from '../../gpu_test.js';
import { align } from '../../util/math.js';
import { generateTypes, supportedScalarTypes, supportsAtomics } from '../types.js';

export const g = makeTestGroup(GPUTest);

const kMaxU32 = 0xffff_ffff;
const kMaxI32 = 0x7fff_ffff;
const kMinI32 = -0x8000_0000;

/**
 * Wraps the provided source into a harness that checks calling `runTest()` returns 0.
 *
 * Non-test bindings are in bind group 1, including:
 * - `constants.zero`: a dynamically-uniform `0u` value.
 */
async function runShaderTest(
t,
enables,
stage,
testSource,
layout,
testBindings,
dynamicOffsets)
{
  assert(stage === GPUShaderStage.COMPUTE, 'Only know how to deal with compute for now');

  // Contains just zero (for now).
  const constantsBuffer = t.createBufferTracked({ size: 4, usage: GPUBufferUsage.UNIFORM });

  const resultBuffer = t.createBufferTracked({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  });

  const source = `${enables}
struct Constants {
  zero: u32
};
@group(1) @binding(0) var<uniform> constants: Constants;

struct Result {
  value: u32
};
@group(1) @binding(1) var<storage, read_write> result: Result;

${testSource}

@compute @workgroup_size(1)
fn main() {
  _ = constants.zero; // Ensure constants buffer is statically-accessed
  result.value = runTest();
}`;

  t.debug(source);
  const module = t.device.createShaderModule({ code: source });
  const pipeline = await t.device.createComputePipelineAsync({
    layout,
    compute: { module, entryPoint: 'main' }
  });

  const group = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(1),
    entries: [
    { binding: 0, resource: { buffer: constantsBuffer } },
    { binding: 1, resource: { buffer: resultBuffer } }]

  });

  const testGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: testBindings
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, testGroup, dynamicOffsets);
  pass.setBindGroup(1, group);
  pass.dispatchWorkgroups(1);
  pass.end();

  t.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(resultBuffer, new Uint32Array([0]));
}

/** Fill an ArrayBuffer with sentinel values, except clear a region to zero. */
function testFillArrayBuffer(
array,
type,
{ zeroByteStart, zeroByteCount })
{
  const constructor = { u32: Uint32Array, i32: Int32Array, f16: Float16Array, f32: Float32Array }[
  type];

  assert(zeroByteCount % constructor.BYTES_PER_ELEMENT === 0);
  new constructor(array).fill(42);
  new constructor(array, zeroByteStart, zeroByteCount / constructor.BYTES_PER_ELEMENT).fill(0);
}

/**
 * Generate a bunch of indexable types (vec, mat, sized/unsized array) for testing.
 */

g.test('linear_memory').
desc(
  `For each indexable data type (vec, mat, sized/unsized array, of various scalar types), attempts
    to access (read, write, atomic load/store) a region of memory (buffer or internal) at various
    (signed/unsigned) indices. Checks that the accesses conform to robust access (OOB reads only
    return bound memory, OOB writes don't write OOB).

    TODO: Test in/out storage classes.
    TODO: Test vertex and fragment stages.
    TODO: Test using a dynamic offset instead of a static offset into uniform/storage bindings.
    TODO: Test types like vec2<atomic<i32>>, if that's allowed.
    TODO: Test exprIndexAddon as constexpr.
    TODO: Test exprIndexAddon as pipeline-overridable constant expression.
    TODO: Adjust test logic to support array of f16 in the uniform address space
  `
).
params((u) =>
u.
combineWithParams([
{ addressSpace: 'storage', storageMode: 'read', access: 'read', dynamicOffset: false },
{
  addressSpace: 'storage',
  storageMode: 'read_write',
  access: 'read',
  dynamicOffset: false
},
{
  addressSpace: 'storage',
  storageMode: 'read_write',
  access: 'write',
  dynamicOffset: false
},
{ addressSpace: 'storage', storageMode: 'read', access: 'read', dynamicOffset: true },
{ addressSpace: 'storage', storageMode: 'read_write', access: 'read', dynamicOffset: true },
{
  addressSpace: 'storage',
  storageMode: 'read_write',
  access: 'write',
  dynamicOffset: true
},
{ addressSpace: 'uniform', access: 'read', dynamicOffset: false },
{ addressSpace: 'uniform', access: 'read', dynamicOffset: true },
{ addressSpace: 'private', access: 'read' },
{ addressSpace: 'private', access: 'write' },
{ addressSpace: 'function', access: 'read' },
{ addressSpace: 'function', access: 'write' },
{ addressSpace: 'workgroup', access: 'read' },
{ addressSpace: 'workgroup', access: 'write' }]
).
combineWithParams([
{ containerType: 'array' },
{ containerType: 'matrix' },
{ containerType: 'vector' }]
).
combineWithParams([
{ shadowingMode: 'none' },
{ shadowingMode: 'module-scope' },
{ shadowingMode: 'function-scope' }]
).
expand('isAtomic', (p) => supportsAtomics(p) ? [false, true] : [false]).
expand('baseType', supportedScalarTypes).
beginSubcases().
expandWithParams(generateTypes)
).
beforeAllSubcases((t) => {
  if (t.params.baseType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const {
    addressSpace,
    storageMode,
    access,
    dynamicOffset,
    isAtomic,
    containerType,
    baseType,
    type,
    shadowingMode,
    _kTypeInfo
  } = t.params;

  assert(_kTypeInfo !== undefined, 'not an indexable type');
  assert('arrayLength' in _kTypeInfo);

  if (baseType === 'f16' && addressSpace === 'uniform' && containerType === 'array') {
    // Array elements must be aligned to 16 bytes, but the logic in generateTypes
    // creates an array of vec4 of the baseType. But for f16 that's only 8 bytes.
    // We would need to write more complex logic for that.
    t.skip('Test logic does not handle array of f16 in the uniform address space');
  }

  let usesCanary = false;
  let globalSource = '';
  let testFunctionSource = '';
  const testBufferSize = 512;
  const bufferBindingOffset = 256;
  /** Undefined if no buffer binding is needed */
  let bufferBindingSize = undefined;

  // Declare the data that will be accessed to check robust access, as a buffer or a struct
  // in the global scope or inside the test function itself.
  const structDecl = `
struct S {
  startCanary: array<u32, 10>,
  data: ${type},
  endCanary: array<u32, 10>,
};`;

  const testGroupBGLEntires = [];
  switch (addressSpace) {
    case 'uniform':
    case 'storage':
      {
        assert(_kTypeInfo.layout !== undefined);
        const layout = _kTypeInfo.layout;
        bufferBindingSize = align(layout.size, layout.alignment);
        const qualifiers = addressSpace === 'storage' ? `storage, ${storageMode}` : addressSpace;
        globalSource += `
struct TestData {
  data: ${type},
};
@group(0) @binding(0) var<${qualifiers}> s: TestData;`;

        testGroupBGLEntires.push({
          binding: 0,
          visibility: GPUShaderStage.COMPUTE,
          buffer: {
            type:
            addressSpace === 'uniform' ?
            'uniform' :
            storageMode === 'read' ?
            'read-only-storage' :
            'storage',
            hasDynamicOffset: dynamicOffset
          }
        });
      }
      break;

    case 'private':
    case 'workgroup':
      usesCanary = true;
      globalSource += structDecl;
      globalSource += `var<${addressSpace}> s: S;`;
      break;

    case 'function':
      usesCanary = true;
      globalSource += structDecl;
      testFunctionSource += 'var s: S;';
      break;
  }

  // Build the test function that will do the tests.

  // If we use a local canary declared in the shader, initialize it.
  if (usesCanary) {
    testFunctionSource += `
  for (var i = 0u; i < 10u; i = i + 1u) {
    s.startCanary[i] = 0xFFFFFFFFu;
    s.endCanary[i] = 0xFFFFFFFFu;
  }`;
  }

  /** Returns a different number each time, kind of like a `__LINE__` to ID the failing check. */
  const nextErrorReturnValue = (() => {
    let errorReturnValue = 0x1000;
    return () => {
      ++errorReturnValue;
      return `0x${errorReturnValue.toString(16)}u`;
    };
  })();

  // This is here, instead of in subcases, so only a single shader is needed to test many modes.
  for (const indexSigned of [false, true]) {
    const indicesToTest = indexSigned ?
    [
    // Exactly in bounds (should be OK)
    '0',
    `${_kTypeInfo.arrayLength} - 1`,
    // Exactly out of bounds
    '-1',
    `${_kTypeInfo.arrayLength}`,
    // Far out of bounds
    '-1000000',
    '1000000',
    `${kMinI32}`,
    `${kMaxI32}`] :

    [
    // Exactly in bounds (should be OK)
    '0u',
    `${_kTypeInfo.arrayLength}u - 1u`,
    // Exactly out of bounds
    `${_kTypeInfo.arrayLength}u`,
    // Far out of bounds
    '1000000u',
    `${kMaxU32}u`,
    `${kMaxI32}u`];


    const indexTypeLiteral = indexSigned ? '0' : '0u';
    const indexTypeCast = indexSigned ? 'i32' : 'u32';
    for (const exprIndexAddon of [
    '', // No addon
    ` + ${indexTypeLiteral}`, // Add a literal 0
    ` + ${indexTypeCast}(constants.zero)` // Add a uniform 0
    ]) {
      // Produce the accesses to the variable.
      for (const indexToTest of indicesToTest) {
        testFunctionSource += `
  {
    let index = (${indexToTest})${exprIndexAddon};`;
        const exprZeroElement = `${_kTypeInfo.elementBaseType}()`;
        const exprElement = `s.data[index]`;

        switch (access) {
          case 'read':
            {
              let exprLoadElement = isAtomic ? `atomicLoad(&${exprElement})` : exprElement;
              if (addressSpace === 'uniform' && containerType === 'array') {
                // Scalar types will be wrapped in a vec4 to satisfy array element size
                // requirements for the uniform address space, so we need an additional index
                // accessor expression.
                exprLoadElement += '[0]';
              }
              let condition = `${exprLoadElement} != ${exprZeroElement}`;
              if (containerType === 'matrix') condition = `any(${condition})`;
              testFunctionSource += `
    if (${condition}) { return ${nextErrorReturnValue()}; }`;
            }
            break;

          case 'write':
            if (isAtomic) {
              testFunctionSource += `
    atomicStore(&s.data[index], ${exprZeroElement});`;
            } else {
              testFunctionSource += `
    s.data[index] = ${exprZeroElement};`;
            }
            break;
        }
        testFunctionSource += `
  }`;
      }
    }
  }

  // Check that the canaries haven't been modified
  if (usesCanary) {
    testFunctionSource += `
  for (var i = 0u; i < 10u; i = i + 1u) {
    if (s.startCanary[i] != 0xFFFFFFFFu) {
      return ${nextErrorReturnValue()};
    }
    if (s.endCanary[i] != 0xFFFFFFFFu) {
      return ${nextErrorReturnValue()};
    }
  }`;
  }

  // Shadowing case declarations
  let moduleScopeShadowDecls = '';
  let functionScopeShadowDecls = '';

  switch (shadowingMode) {
    case 'module-scope':
      // Shadow the builtins likely used by robustness as module-scope variables
      moduleScopeShadowDecls = `
var<private> min = 0;
var<private> max = 0;
var<private> arrayLength = 0;
`;
      // Make sure that these are referenced by the function.
      // This ensures that compilers don't strip away unused variables.
      functionScopeShadowDecls = `
  _ = min;
  _ = max;
  _ = arrayLength;
`;
      break;
    case 'function-scope':
      // Shadow the builtins likely used by robustness as function-scope variables
      functionScopeShadowDecls = `
  let min = 0;
  let max = 0;
  let arrayLength = 0;
`;
      break;
  }

  // Run the test

  // First aggregate the test source
  const testSource = `
${globalSource}
${moduleScopeShadowDecls}

fn runTest() -> u32 {
  ${functionScopeShadowDecls}
  ${testFunctionSource}
  return 0u;
}`;

  const layout = t.device.createPipelineLayout({
    bindGroupLayouts: [
    t.device.createBindGroupLayout({
      entries: testGroupBGLEntires
    }),
    t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: 'uniform'
        }
      },
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: 'storage'
        }
      }]

    })]

  });

  const enables = t.params.baseType === 'f16' ? 'enable f16;' : '';

  // Run it.
  if (bufferBindingSize !== undefined && baseType !== 'bool') {
    const expectedData = new ArrayBuffer(testBufferSize);
    const bufferBindingEnd = bufferBindingOffset + bufferBindingSize;
    testFillArrayBuffer(expectedData, baseType, {
      zeroByteStart: bufferBindingOffset,
      zeroByteCount: bufferBindingSize
    });

    // Create a buffer that contains zeroes in the allowed access area, and 42s everywhere else.
    const testBuffer = t.makeBufferWithContents(
      new Uint8Array(expectedData),
      GPUBufferUsage.COPY_SRC |
      GPUBufferUsage.UNIFORM |
      GPUBufferUsage.STORAGE |
      GPUBufferUsage.COPY_DST
    );

    // Run the shader, accessing the buffer.
    await runShaderTest(
      t,
      enables,
      GPUShaderStage.COMPUTE,
      testSource,
      layout,
      [
      {
        binding: 0,
        resource: {
          buffer: testBuffer,
          offset: dynamicOffset ? 0 : bufferBindingOffset,
          size: bufferBindingSize
        }
      }],

      dynamicOffset ? [bufferBindingOffset] : undefined
    );

    // Check that content of the buffer outside of the allowed area didn't change.
    const expectedBytes = new Uint8Array(expectedData);
    t.expectGPUBufferValuesEqual(testBuffer, expectedBytes.subarray(0, bufferBindingOffset), 0);
    t.expectGPUBufferValuesEqual(
      testBuffer,
      expectedBytes.subarray(bufferBindingEnd, testBufferSize),
      bufferBindingEnd
    );
  } else {
    await runShaderTest(t, enables, GPUShaderStage.COMPUTE, testSource, layout, []);
  }
});