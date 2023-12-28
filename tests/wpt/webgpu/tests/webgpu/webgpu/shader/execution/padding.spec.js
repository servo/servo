/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for preservation of padding bytes in structures and arrays.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { iterRange } from '../../../common/util/util.js';
import { GPUTest } from '../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

/**
 * Run a shader and check that the buffer output matches expectations.
 *
 * @param t The test object
 * @param wgsl The shader source
 * @param expected The array of expected values after running the shader
 */
function runShaderTest(t, wgsl, expected) {
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  // Allocate a buffer and fill it with 0xdeadbeef words.
  const outputBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(expected.length, (_i) => 0xdeadbeef)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  );
  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: { buffer: outputBuffer } }]
  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Check that only the non-padding bytes were modified.
  t.expectGPUBufferValuesEqual(outputBuffer, expected);
}

g.test('struct_implicit').
desc(
  `Test that padding bytes in between structure members are preserved.

     This test defines a structure that has implicit padding and creates a read-write storage
     buffer with that structure type. The shader assigns the whole variable at once, and we
     then test that data in the padding bytes was preserved.
    `
).
fn((t) => {
  const wgsl = `
      struct S {
        a : u32,
        // 12 bytes of padding
        b : vec3<u32>,
        // 4 bytes of padding
        c : vec2<u32>,
        // 8 bytes of padding
      }
      @group(0) @binding(0) var<storage, read_write> buffer : S;

      @compute @workgroup_size(1)
      fn main() {
        buffer = S(0x12345678, vec3(0xabcdef01), vec2(0x98765432));
      }
    `;
  runShaderTest(
    t,
    wgsl,
    new Uint32Array([
    // a : u32
    0x12345678, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef,
    // b : vec3<u32>
    0xabcdef01, 0xabcdef01, 0xabcdef01, 0xdeadbeef,
    // c : vec2<u32>
    0x98765432, 0x98765432, 0xdeadbeef, 0xdeadbeef]
    )
  );
});

g.test('struct_explicit').
desc(
  `Test that padding bytes in between structure members are preserved.

     This test defines a structure with explicit padding attributes and creates a read-write storage
     buffer with that structure type. The shader assigns the whole variable at once, and we
     then test that data in the padding bytes was preserved.
    `
).
fn((t) => {
  const wgsl = `
      struct S {
        a : u32,
        // 12 bytes of padding
        @align(16) @size(20) b : u32,
        // 16 bytes of padding
        @size(12) c : u32,
        // 8 bytes of padding
      }
      @group(0) @binding(0) var<storage, read_write> buffer : S;

      @compute @workgroup_size(1)
      fn main() {
        buffer = S(0x12345678, 0xabcdef01, 0x98765432);
      }
    `;
  runShaderTest(
    t,
    wgsl,
    new Uint32Array([
    // a : u32
    0x12345678, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef,
    // @align(16) @size(20) b : u32
    0xabcdef01, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef,
    // @size(12) c : u32
    0x98765432, 0xdeadbeef, 0xdeadbeef]
    )
  );
});

g.test('struct_nested').
desc(
  `Test that padding bytes in nested structures are preserved.

     This test defines a set of nested structures that have padding and creates a read-write storage
     buffer with the root structure type. The shader assigns the whole variable at once, and we
     then test that data in the padding bytes was preserved.
    `
).
fn((t) => {
  const wgsl = `
      // Size of S1 is 48 bytes.
      // Alignment of S1 is 16 bytes.
      struct S1 {
        a : u32,
        // 12 bytes of padding
        b : vec3<u32>,
        // 4 bytes of padding
        c : vec2<u32>,
        // 8 bytes of padding
      }

      // Size of S2 is 112 bytes.
      // Alignment of S2 is 48 bytes.
      struct S2 {
        a2 : u32,
        // 12 bytes of padding
        b2 : S1,
        c2 : S1,
      }

      // Size of S3 is 144 bytes.
      // Alignment of S3 is 48 bytes.
      struct S3 {
        a3 : S1,
        b3 : S2,
        c3 : S2,
      }

      @group(0) @binding(0) var<storage, read_write> buffer : S3;

      @compute @workgroup_size(1)
      fn main() {
        buffer = S3();
      }
    `;
  runShaderTest(
    t,
    wgsl,
    new Uint32Array([
    // a3 : S1
    // a3.a1 : u32
    0x00000000, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef,
    // a3.b1 : vec3<u32>
    0x00000000, 0x00000000, 0x00000000, 0xdeadbeef,
    // a3.c1 : vec2<u32>
    0x00000000, 0x00000000, 0xdeadbeef, 0xdeadbeef,

    // b3 : S2
    // b3.a2 : u32
    0x00000000, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef,
    // b3.b2 : S1
    // b3.b2.a1 : u32
    0x00000000, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef,
    // b3.b2.b1 : vec3<u32>
    0x00000000, 0x00000000, 0x00000000, 0xdeadbeef,
    // b3.b2.c1 : vec2<u32>
    0x00000000, 0x00000000, 0xdeadbeef, 0xdeadbeef,
    // b3.c2 : S1
    // b3.c2.a1 : u32
    0x00000000, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef,
    // b3.c2.b1 : vec3<u32>
    0x00000000, 0x00000000, 0x00000000, 0xdeadbeef,
    // b3.c2.c1 : vec2<u32>
    0x00000000, 0x00000000, 0xdeadbeef, 0xdeadbeef,

    // c3 : S2
    // c3.a2 : u32
    0x00000000, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef,
    // c3.b2 : S1
    // c3.b2.a1 : u32
    0x00000000, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef,
    // c3.b2.b1 : vec3<u32>
    0x00000000, 0x00000000, 0x00000000, 0xdeadbeef,
    // c3.b2.c1 : vec2<u32>
    0x00000000, 0x00000000, 0xdeadbeef, 0xdeadbeef,
    // c3.c2 : S1
    // c3.c2.a1 : u32
    0x00000000, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef,
    // c3.c2.b1 : vec3<u32>
    0x00000000, 0x00000000, 0x00000000, 0xdeadbeef,
    // c3.c2.c1 : vec2<u32>
    0x00000000, 0x00000000, 0xdeadbeef, 0xdeadbeef]
    )
  );
});

g.test('array_of_vec3').
desc(
  `Test that padding bytes in between array elements are preserved.

     This test defines creates a read-write storage buffer with type array<vec3, 4>. The shader
     assigns the whole variable at once, and we then test that data in the padding bytes was
     preserved.
    `
).
fn((t) => {
  const wgsl = `
      @group(0) @binding(0) var<storage, read_write> buffer : array<vec3<u32>, 4>;

      @compute @workgroup_size(1)
      fn main() {
        buffer = array<vec3<u32>, 4>(
          vec3(0x12345678),
          vec3(0xabcdef01),
          vec3(0x98765432),
          vec3(0x0f0f0f0f),
        );
      }
    `;
  runShaderTest(
    t,
    wgsl,
    new Uint32Array([
    // buffer[0]
    0x12345678, 0x12345678, 0x12345678, 0xdeadbeef,
    // buffer[1]
    0xabcdef01, 0xabcdef01, 0xabcdef01, 0xdeadbeef,
    // buffer[2]
    0x98765432, 0x98765432, 0x98765432, 0xdeadbeef,
    // buffer[2]
    0x0f0f0f0f, 0x0f0f0f0f, 0x0f0f0f0f, 0xdeadbeef]
    )
  );
});

g.test('array_of_struct').
desc(
  `Test that padding bytes in between array elements are preserved.

     This test defines creates a read-write storage buffer with type array<S, 4>, where S is a
     structure that contains padding bytes. The shader assigns the whole variable at once, and we
     then test that data in the padding bytes was preserved.
    `
).
fn((t) => {
  const wgsl = `
      struct S {
        a : u32,
        b : vec3<u32>,
      }
      @group(0) @binding(0) var<storage, read_write> buffer : array<S, 3>;

      @compute @workgroup_size(1)
      fn main() {
        buffer = array<S, 3>(
          S(0x12345678, vec3(0x0f0f0f0f)),
          S(0xabcdef01, vec3(0x7c7c7c7c)),
          S(0x98765432, vec3(0x18181818)),
        );
      }
    `;
  runShaderTest(
    t,
    wgsl,
    new Uint32Array([
    // buffer[0]
    0x12345678, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef, 0x0f0f0f0f, 0x0f0f0f0f, 0x0f0f0f0f,
    0xdeadbeef,
    // buffer[1]
    0xabcdef01, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef, 0x7c7c7c7c, 0x7c7c7c7c, 0x7c7c7c7c,
    0xdeadbeef,
    // buffer[2]
    0x98765432, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef, 0x18181818, 0x18181818, 0x18181818,
    0xdeadbeef]
    )
  );
});

g.test('vec3').
desc(
  `Test padding bytes are preserved when assigning to a variable of type vec3 (without a struct).
    `
).
fn((t) => {
  const wgsl = `
      @group(0) @binding(0) var<storage, read_write> buffer : vec3<u32>;

      @compute @workgroup_size(1)
      fn main() {
        buffer = vec3<u32>(0x12345678, 0xabcdef01, 0x98765432);
      }
    `;
  runShaderTest(t, wgsl, new Uint32Array([0x12345678, 0xabcdef01, 0x98765432, 0xdeadbeef]));
});

g.test('matCx3').
desc(
  `Test padding bytes are preserved when assigning to a variable of type matCx3.
    `
).
params((u) =>
u.
combine('columns', [2, 3, 4]).
combine('use_struct', [true, false]).
beginSubcases()
).
fn((t) => {
  const cols = t.params.columns;
  const wgsl = `
      alias Mat = mat${cols}x3<f32>;
      ${t.params.use_struct ? `struct S { m : Mat } alias Type = S;` : `alias Type = Mat;`}
      @group(0) @binding(0) var<storage, read_write> buffer : Type;

      @compute @workgroup_size(1)
      fn main() {
        var m : Mat;
        for (var c = 0u; c < ${cols}; c++) {
          m[c] = vec3(f32(c*3 + 1), f32(c*3 + 2), f32(c*3 + 3));
        }
        buffer = Type(m);
      }
    `;
  const f_values = new Float32Array(cols * 4);
  const u_values = new Uint32Array(f_values.buffer);
  for (let c = 0; c < cols; c++) {
    f_values[c * 4 + 0] = c * 3 + 1;
    f_values[c * 4 + 1] = c * 3 + 2;
    f_values[c * 4 + 2] = c * 3 + 3;
    u_values[c * 4 + 3] = 0xdeadbeef;
  }
  runShaderTest(t, wgsl, u_values);
});

g.test('array_of_matCx3').
desc(
  `Test that padding bytes in between array elements are preserved.

     This test defines creates a read-write storage buffer with type array<matCx3<f32>, 4>. The
     shader assigns the whole variable at once, and we then test that data in the padding bytes was
     preserved.
    `
).
params((u) =>
u.
combine('columns', [2, 3, 4]).
combine('use_struct', [true, false]).
beginSubcases()
).
fn((t) => {
  const cols = t.params.columns;
  const wgsl = `
    alias Mat = mat${cols}x3<f32>;
    ${t.params.use_struct ? `struct S { m : Mat } alias Type = S;` : `alias Type = Mat;`}
    @group(0) @binding(0) var<storage, read_write> buffer : array<Type, 4>;

    @compute @workgroup_size(1)
    fn main() {
      var m : Mat;
      for (var c = 0u; c < ${cols}; c++) {
        m[c] = vec3(f32(c*3 + 1), f32(c*3 + 2), f32(c*3 + 3));
      }
      buffer = array<Type, 4>(Type(m), Type(m * 2), Type(m * 3), Type(m * 4));
    }
  `;
  const f_values = new Float32Array(cols * 4 * 4);
  const u_values = new Uint32Array(f_values.buffer);
  for (let i = 0; i < 4; i++) {
    for (let c = 0; c < cols; c++) {
      f_values[i * (cols * 4) + c * 4 + 0] = (c * 3 + 1) * (i + 1);
      f_values[i * (cols * 4) + c * 4 + 1] = (c * 3 + 2) * (i + 1);
      f_values[i * (cols * 4) + c * 4 + 2] = (c * 3 + 3) * (i + 1);
      u_values[i * (cols * 4) + c * 4 + 3] = 0xdeadbeef;
    }
  }
  runShaderTest(t, wgsl, u_values);
});