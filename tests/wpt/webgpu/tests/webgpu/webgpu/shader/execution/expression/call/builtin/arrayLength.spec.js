/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for the 'arrayLength' builtin function.

fn arrayLength(e: ptr<storage,array<T>> ) -> u32
Returns the number of elements in the runtime-sized array.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { align } from '../../../../../util/math.js';

export const g = makeTestGroup(GPUTest);

// List of array element types to test.
const kTestTypes = [
{ type: 'u32', stride: 4 },
{ type: 'i32', stride: 4 },
{ type: 'f32', stride: 4 },
{ type: 'f16', stride: 2 },
{ type: 'vec2<u32>', stride: 8 },
{ type: 'vec2<i32>', stride: 8 },
{ type: 'vec2<f32>', stride: 8 },
{ type: 'vec2<f16>', stride: 4 },
{ type: 'vec3<u32>', stride: 16 },
{ type: 'vec3<i32>', stride: 16 },
{ type: 'vec3<f32>', stride: 16 },
{ type: 'vec3<f16>', stride: 8 },
{ type: 'vec4<u32>', stride: 16 },
{ type: 'vec4<i32>', stride: 16 },
{ type: 'vec4<f32>', stride: 16 },
{ type: 'vec4<f16>', stride: 8 },
{ type: 'mat2x2<f32>', stride: 16 },
{ type: 'mat2x3<f32>', stride: 32 },
{ type: 'mat2x4<f32>', stride: 32 },
{ type: 'mat3x2<f32>', stride: 24 },
{ type: 'mat3x3<f32>', stride: 48 },
{ type: 'mat3x4<f32>', stride: 48 },
{ type: 'mat4x2<f32>', stride: 32 },
{ type: 'mat4x3<f32>', stride: 64 },
{ type: 'mat4x4<f32>', stride: 64 },
{ type: 'mat2x2<f16>', stride: 8 },
{ type: 'mat2x3<f16>', stride: 16 },
{ type: 'mat2x4<f16>', stride: 16 },
{ type: 'mat3x2<f16>', stride: 12 },
{ type: 'mat3x3<f16>', stride: 24 },
{ type: 'mat3x4<f16>', stride: 24 },
{ type: 'mat4x2<f16>', stride: 16 },
{ type: 'mat4x3<f16>', stride: 32 },
{ type: 'mat4x4<f16>', stride: 32 },
{ type: 'atomic<u32>', stride: 4 },
{ type: 'atomic<i32>', stride: 4 },
{ type: 'array<u32,4>', stride: 16 },
{ type: 'array<i32,4>', stride: 16 },
{ type: 'array<f32,4>', stride: 16 },
{ type: 'array<f16,4>', stride: 8 },
// Structures - see declarations below.
{ type: 'ElemStruct', stride: 4 },
{ type: 'ElemStruct_ImplicitPadding', stride: 16 },
{ type: 'ElemStruct_ExplicitPadding', stride: 32 }];


// Declarations for structures used as array element types.
const kWgslStructures = `
struct ElemStruct { a : u32 }
struct ElemStruct_ImplicitPadding { a : vec3<u32> }
struct ElemStruct_ExplicitPadding { @align(32) a : u32 }
`;

/**
 * Run a shader and check that the array length is correct.
 *
 * @param t The test object
 * @param wgsl The shader source
 * @param stride The stride in bytes of the array element type
 * @param offset The offset in bytes of the array from the start of the binding
 * @param buffer_size The size in bytes of the buffer to allocate
 * @param binding_size The size in bytes of the binding
 * @param binding_offset The offset in bytes of the binding
 * @param expected The array of expected values after running the shader
 */
function runShaderTest(
t,
wgsl,
stride,
offset,
buffer_size,
binding_size,
binding_offset)
{
  // Create the compute pipeline.
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  // Create the buffer that will contain the runtime-sized array.
  const buffer = t.device.createBuffer({
    size: buffer_size,
    usage: GPUBufferUsage.STORAGE
  });

  // Create the buffer that will receive the array length.
  const lengthBuffer = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  // Set up bindings.
  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: { buffer, size: binding_size, offset: binding_offset } },
    { binding: 1, resource: { buffer: lengthBuffer } }]

  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Check the length.
  const length = (binding_size - offset) / stride;
  t.expectGPUBufferValuesEqual(lengthBuffer, new Uint32Array([length]));
}

/**
 * Test if a WGSL type string require using f16 extension.
 *
 * @param test_type The wgsl type for testing
 */
function typeRequiresF16(test_type) {
  return test_type.includes('f16');
}

/**
 * Generate the necessary wgsl header for tested type, especially for f16
 *
 * @param test_type The wgsl type for testing
 */
function shaderHeader(test_type) {
  return typeRequiresF16(test_type) ? 'enable f16;\n\n' : '';
}

g.test('single_element').
specURL('https://www.w3.org/TR/WGSL/#arrayLength-builtin').
desc(
  `Test the arrayLength() builtin with a binding that is just large enough for a single element.

     Test parameters:
     - type: The WGSL type to use as the array element type.
     - stride: The stride in bytes of the array element type.
    `
).
params((u) => u.combineWithParams(kTestTypes)).
beforeAllSubcases((t) => {
  if (typeRequiresF16(t.params.type)) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const wgsl =
  shaderHeader(t.params.type) +
  kWgslStructures +
  `
      @group(0) @binding(0) var<storage, read_write> buffer : array<${t.params.type}>;
      @group(0) @binding(1) var<storage, read_write> length : u32;
      @compute @workgroup_size(1)
      fn main() {
        length = arrayLength(&buffer);
      }
    `;
  let buffer_size = t.params.stride;
  // Ensure that binding size is multiple of 4.
  buffer_size = buffer_size + (~buffer_size + 1 & 3);
  runShaderTest(t, wgsl, t.params.stride, 0, buffer_size, buffer_size, 0);
});

g.test('multiple_elements').
specURL('https://www.w3.org/TR/WGSL/#arrayLength-builtin').
desc(
  `Test the arrayLength() builtin with a binding that is large enough for multiple elements.

     We test sizes that are not exact multiples of the array element strides, to test that the
     length is correctly floored to the next whole element.

     Test parameters:
     - buffer_size: The size in bytes of the buffer.
     - type: The WGSL type to use as the array element type.
     - stride: The stride in bytes of the array element type.
    `
).
params((u) =>
u.combine('buffer_size', [640, 1004, 1048576]).combineWithParams(kTestTypes)
).
beforeAllSubcases((t) => {
  if (typeRequiresF16(t.params.type)) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const wgsl =
  shaderHeader(t.params.type) +
  kWgslStructures +
  `
      @group(0) @binding(0) var<storage, read_write> buffer : array<${t.params.type}>;
      @group(0) @binding(1) var<storage, read_write> length : u32;
      @compute @workgroup_size(1)
      fn main() {
        length = arrayLength(&buffer);
      }
    `;
  runShaderTest(t, wgsl, t.params.stride, 0, t.params.buffer_size, t.params.buffer_size, 0);
});

g.test('struct_member').
specURL('https://www.w3.org/TR/WGSL/#arrayLength-builtin').
desc(
  `Test the arrayLength() builtin with an array that is inside a structure.

     We include offsets that are not exact multiples of the array element strides, to test that
     the length is correctly floored to the next whole element.

     Test parameters:
     - member_offset: The offset (in bytes) of the array member from the start of the struct.
     - type: The WGSL type to use as the array element type.
     - stride: The stride in bytes of the array element type.
    `
).
params((u) => u.combine('member_offset', [0, 4, 20]).combineWithParams(kTestTypes)).
beforeAllSubcases((t) => {
  if (typeRequiresF16(t.params.type)) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const member_offset = align(t.params.member_offset, t.params.stride);
  const wgsl =
  shaderHeader(t.params.type) +
  kWgslStructures +
  `
      alias ArrayType = array<${t.params.type}>;
      struct Struct {
        ${t.params.member_offset > 0 ? `@size(${member_offset}) padding : u32,` : ``}
        arr : ArrayType,
      }
      @group(0) @binding(0) var<storage, read_write> buffer : Struct;
      @group(0) @binding(1) var<storage, read_write> length : u32;
      @compute @workgroup_size(1)
      fn main() {
        length = arrayLength(&buffer.arr);
      }
    `;
  const buffer_size = 1048576;
  runShaderTest(t, wgsl, t.params.stride, member_offset, buffer_size, buffer_size, 0);
});

g.test('binding_subregion').
specURL('https://www.w3.org/TR/WGSL/#arrayLength-builtin').
desc(
  `Test the arrayLength() builtin when used with a binding that starts at a non-zero offset and
     does not fill the entire buffer.
    `
).
fn((t) => {
  const wgsl = `
      @group(0) @binding(0) var<storage, read_write> buffer : array<vec3<f32>>;
      @group(0) @binding(1) var<storage, read_write> length : u32;
      @compute @workgroup_size(1)
      fn main() {
        length = arrayLength(&buffer);
      }
    `;
  const stride = 16;
  const buffer_size = 1024;
  const binding_size = 640;
  const binding_offset = 256;
  runShaderTest(t, wgsl, stride, 0, buffer_size, binding_size, binding_offset);
});

g.test('read_only').
specURL('https://www.w3.org/TR/WGSL/#arrayLength-builtin').
desc(
  `Test the arrayLength() builtin when used with a read-only storage buffer.
    `
).
fn((t) => {
  const wgsl = `
      @group(0) @binding(0) var<storage, read> buffer : array<vec3<f32>>;
      @group(0) @binding(1) var<storage, read_write> length : u32;
      @compute @workgroup_size(1)
      fn main() {
        length = arrayLength(&buffer);
      }
    `;
  const stride = 16;
  const buffer_size = 1024;
  runShaderTest(t, wgsl, stride, 0, buffer_size, buffer_size, 0);
});