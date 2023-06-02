/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Test the shared use of structures containing entry point IO attributes`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';
import { checkElementsEqual } from '../../../util/check_contents.js';

export const g = makeTestGroup(TextureTestMixin(GPUTest));

g.test('shared_with_buffer')
  .desc(
    `Test sharing an entry point IO struct with a buffer.

     This test defines a structure that contains both builtin attributes and layout attributes,
     and uses that structure as both an entry point input and the store type of a storage buffer.
     The builtin attributes should be ignored when used for the storage buffer, and the layout
     attributes should be ignored when used as an entry point IO parameter.
    `
  )
  .fn(t => {
    // Set the dispatch parameters such that we get some interesting (non-zero) built-in variables.
    const wgsize = new Uint32Array([8, 4, 2]);
    const numGroups = new Uint32Array([4, 2, 8]);

    // Pick a single invocation to copy the input structure to the output buffer.
    const targetLocalIndex = 13;
    const targetGroup = new Uint32Array([2, 1, 5]);

    // The test shader defines a structure that contains members decorated with built-in variable
    // attributes, and also layout attributes for the storage buffer.
    const wgsl = `
      struct S {
        /* byte offset:  0 */ @size(32)  @builtin(workgroup_id) group_id : vec3<u32>,
        /* byte offset: 32 */            @builtin(local_invocation_index) local_index : u32,
        /* byte offset: 64 */ @align(64) @builtin(num_workgroups) numGroups : vec3<u32>,
      };

      @group(0) @binding(0)
      var<storage, read_write> outputs : S;

      @compute @workgroup_size(${wgsize[0]}, ${wgsize[1]}, ${wgsize[2]})
      fn main(inputs : S) {
        if (inputs.group_id.x == ${targetGroup[0]}u &&
            inputs.group_id.y == ${targetGroup[1]}u &&
            inputs.group_id.z == ${targetGroup[2]}u &&
            inputs.local_index == ${targetLocalIndex}u) {
          outputs = inputs;
        }
      }
    `;

    const pipeline = t.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: t.device.createShaderModule({ code: wgsl }),
        entryPoint: 'main',
      },
    });

    // Allocate a buffer to hold the output structure.
    const bufferNumElements = 32;
    const outputBuffer = t.device.createBuffer({
      size: bufferNumElements * Uint32Array.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    const bindGroup = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer: outputBuffer } }],
    });

    // Run the shader.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(numGroups[0], numGroups[1], numGroups[2]);
    pass.end();
    t.queue.submit([encoder.finish()]);

    // Check the output values.
    const checkOutput = outputs => {
      if (checkElementsEqual(outputs.slice(0, 3), targetGroup)) {
        return new Error(
          `group_id comparison failed\n` +
            `    expected: ${targetGroup}\n` +
            `    got:      ${outputs.slice(0, 3)}`
        );
      }
      if (outputs[8] !== targetLocalIndex) {
        return new Error(
          `local_index comparison failed\n` +
            `    expected: ${targetLocalIndex}\n` +
            `    got:      ${outputs[8]}`
        );
      }
      if (checkElementsEqual(outputs.slice(16, 19), numGroups)) {
        return new Error(
          `numGroups comparison failed\n` +
            `    expected: ${numGroups}\n` +
            `    got:      ${outputs.slice(16, 19)}`
        );
      }
      return undefined;
    };
    t.expectGPUBufferValuesPassCheck(outputBuffer, outputData => checkOutput(outputData), {
      type: Uint32Array,
      typedLength: bufferNumElements,
    });
  });

g.test('shared_between_stages')
  .desc(
    `Test sharing an entry point IO struct between different pipeline stages.

     This test defines an entry point IO structure, and uses it as both the output of a vertex
     shader and the input to a fragment shader.
    `
  )
  .fn(t => {
    const size = [31, 31];
    const wgsl = `
      struct Interface {
        @builtin(position) position : vec4<f32>,
        @location(0) color : f32,
      };

      var<private> vertices : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
        vec2<f32>(-0.7, -0.7),
        vec2<f32>( 0.0,  0.7),
        vec2<f32>( 0.7, -0.7),
      );

      @vertex
      fn vert_main(@builtin(vertex_index) index : u32) -> Interface {
        return Interface(vec4<f32>(vertices[index], 0.0, 1.0), 1.0);
      }

      @fragment
      fn frag_main(inputs : Interface) -> @location(0) vec4<f32> {
        // Toggle red vs green based on the x position.
        var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        if (inputs.position.x > f32(${size[0] / 2})) {
          color.r = inputs.color;
        } else {
          color.g = inputs.color;
        }
        return color;
      }
    `;

    // Set up the render pipeline.
    const module = t.device.createShaderModule({ code: wgsl });
    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module,
        entryPoint: 'vert_main',
      },
      fragment: {
        module,
        entryPoint: 'frag_main',
        targets: [
          {
            format: 'rgba8unorm',
          },
        ],
      },
    });

    // Draw a red triangle.
    const renderTarget = t.device.createTexture({
      size,
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      format: 'rgba8unorm',
    });
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view: renderTarget.createView(),
          clearValue: [0, 0, 0, 0],
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
    });
    pass.setPipeline(pipeline);
    pass.draw(3);
    pass.end();
    t.queue.submit([encoder.finish()]);

    // Test a few points to make sure we rendered a half-red/half-green triangle.
    const redPixel = new Uint8Array([255, 0, 0, 255]);
    const greenPixel = new Uint8Array([0, 255, 0, 255]);
    const blackPixel = new Uint8Array([0, 0, 0, 0]);
    t.expectSinglePixelComparisonsAreOkInTexture({ texture: renderTarget }, [
      // Red pixels
      { coord: { x: 16, y: 15 }, exp: redPixel },
      { coord: { x: 16, y: 8 }, exp: redPixel },
      { coord: { x: 22, y: 20 }, exp: redPixel },
      // Green pixels
      { coord: { x: 14, y: 15 }, exp: greenPixel },
      { coord: { x: 14, y: 8 }, exp: greenPixel },
      { coord: { x: 8, y: 20 }, exp: greenPixel },
      // Black pixels
      { coord: { x: 2, y: 2 }, exp: blackPixel },
      { coord: { x: 2, y: 28 }, exp: blackPixel },
      { coord: { x: 28, y: 2 }, exp: blackPixel },
      { coord: { x: 28, y: 28 }, exp: blackPixel },
    ]);
  });

g.test('shared_with_non_entry_point_function')
  .desc(
    `Test sharing an entry point IO struct with a non entry point function.

     This test defines structures that contain builtin and location attributes, and uses those
     structures as parameter and return types for entry point functions and regular functions.
    `
  )
  .fn(t => {
    // The test shader defines structures that contain members decorated with built-in variable
    // attributes and user-defined IO. These structures are passed to and returned from regular
    // functions.
    const wgsl = `
      struct Inputs {
        @builtin(vertex_index) index : u32,
        @location(0) color : vec4<f32>,
      };
      struct Outputs {
        @builtin(position) position : vec4<f32>,
        @location(0) color : vec4<f32>,
      };

      var<private> vertices : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
        vec2<f32>(-0.7, -0.7),
        vec2<f32>( 0.0,  0.7),
        vec2<f32>( 0.7, -0.7),
      );

      fn process(in : Inputs) -> Outputs {
        var out : Outputs;
        out.position = vec4<f32>(vertices[in.index], 0.0, 1.0);
        out.color = in.color;
        return out;
      }

      @vertex
      fn vert_main(inputs : Inputs) -> Outputs {
        return process(inputs);
      }

      @fragment
      fn frag_main(@location(0) color : vec4<f32>) -> @location(0) vec4<f32> {
        return color;
      }
    `;

    // Set up the render pipeline.
    const module = t.device.createShaderModule({ code: wgsl });
    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module,
        entryPoint: 'vert_main',
        buffers: [
          {
            attributes: [
              {
                shaderLocation: 0,
                format: 'float32x4',
                offset: 0,
              },
            ],

            arrayStride: 4 * Float32Array.BYTES_PER_ELEMENT,
          },
        ],
      },
      fragment: {
        module,
        entryPoint: 'frag_main',
        targets: [
          {
            format: 'rgba8unorm',
          },
        ],
      },
    });

    // Draw a triangle.
    // The vertex buffer contains the vertex colors (all red).
    const vertexBuffer = t.makeBufferWithContents(
      new Float32Array([1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0]),
      GPUBufferUsage.VERTEX
    );

    const renderTarget = t.device.createTexture({
      size: [31, 31],
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      format: 'rgba8unorm',
    });
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view: renderTarget.createView(),
          clearValue: [0, 0, 0, 0],
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
    });
    pass.setPipeline(pipeline);
    pass.setVertexBuffer(0, vertexBuffer);
    pass.draw(3);
    pass.end();
    t.queue.submit([encoder.finish()]);

    // Test a few points to make sure we rendered a red triangle.
    const redPixel = new Uint8Array([255, 0, 0, 255]);
    const blackPixel = new Uint8Array([0, 0, 0, 0]);
    t.expectSinglePixelComparisonsAreOkInTexture({ texture: renderTarget }, [
      // Red pixels
      { coord: { x: 15, y: 15 }, exp: redPixel },
      { coord: { x: 15, y: 8 }, exp: redPixel },
      { coord: { x: 8, y: 20 }, exp: redPixel },
      { coord: { x: 22, y: 20 }, exp: redPixel },
      // Black pixels
      { coord: { x: 2, y: 2 }, exp: blackPixel },
      { coord: { x: 2, y: 28 }, exp: blackPixel },
      { coord: { x: 28, y: 2 }, exp: blackPixel },
      { coord: { x: 28, y: 28 }, exp: blackPixel },
    ]);
  });
