/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for the creation of pipeline layouts with null bind group layouts.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUConst } from '../../../constants.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('pipeline_layout_with_null_bind_group_layout,rendering').
desc(
  `
Tests that using a render pipeline created with a pipeline layout that has null bind group layout
works correctly.
`
).
params((u) =>
u.
combine('emptyBindGroupLayoutType', ['Null', 'Undefined', 'Empty']).
combine('emptyBindGroupLayoutIndex', [0, 1, 2, 3])
).
fn((t) => {
  const { emptyBindGroupLayoutType, emptyBindGroupLayoutIndex } = t.params;

  const colors = [
  [0.2, 0, 0, 0.2],
  [0, 0.2, 0, 0.2],
  [0, 0, 0.2, 0.2],
  [0.4, 0, 0, 0.2]];

  const outputColor = [0.0, 0.0, 0.0, 0.0];

  let declarations = '';
  let statement = 'return vec4(0.0, 0.0, 0.0, 0.0)';
  const bindGroupLayouts = [];
  const bindGroups = [];
  for (let bindGroupIndex = 0; bindGroupIndex < 4; ++bindGroupIndex) {
    const bindGroupLayout = t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUConst.ShaderStage.FRAGMENT,
        buffer: {
          type: 'uniform',
          minBindingSize: 16
        }
      }]

    });

    const color = colors[bindGroupIndex];
    const buffer = t.makeBufferWithContents(new Float32Array(color), GPUBufferUsage.UNIFORM);

    // Still create and set the bind group when the corresponding bind group layout in the
    // pipeline is null. The output color should not be affected by the buffer in this bind group
    const bindGroup = t.device.createBindGroup({
      layout: bindGroupLayout,
      entries: [
      {
        binding: 0,
        resource: {
          buffer
        }
      }]

    });
    bindGroups.push(bindGroup);

    // Set `null`, `undefined` or empty bind group layout in `bindGroupLayouts` which is used in
    // the creation of pipeline layout
    if (bindGroupIndex === emptyBindGroupLayoutIndex) {
      switch (emptyBindGroupLayoutType) {
        case 'Null':
          bindGroupLayouts.push(null);
          break;
        case 'Undefined':
          bindGroupLayouts.push(undefined);
          break;
        case 'Empty':
          bindGroupLayouts.push(
            t.device.createBindGroupLayout({
              entries: []
            })
          );
          break;
      }
      continue;
    }

    // Set the uniform buffers used in the shader
    bindGroupLayouts.push(bindGroupLayout);
    declarations += `@group(${bindGroupIndex}) @binding(0) var<uniform> input${bindGroupIndex} : vec4f;\n`;
    statement += ` + input${bindGroupIndex}`;

    // Compute the expected output color
    for (let i = 0; i < color.length; ++i) {
      outputColor[i] += color[i];
    }
  }

  const pipelineLayout = t.device.createPipelineLayout({
    bindGroupLayouts
  });

  const format = 'rgba8unorm';
  const code = `
    ${declarations}
    @vertex
    fn vert_main() -> @builtin(position) vec4f {
        return vec4f(0.0, 0.0, 0.0, 1.0);
    }
    @fragment
    fn frag_main() -> @location(0) vec4f {
        ${statement};
    }
    `;
  const shaderModule = t.device.createShaderModule({
    code
  });
  const renderPipeline = t.device.createRenderPipeline({
    layout: pipelineLayout,
    vertex: {
      module: shaderModule
    },
    fragment: {
      module: shaderModule,
      targets: [
      {
        format
      }]

    },
    primitive: {
      topology: 'point-list'
    }
  });

  const renderTarget = t.createTextureTracked({
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
    size: [1, 1, 1],
    format
  });
  const commandEncoder = t.device.createCommandEncoder();
  const renderPassEncoder = commandEncoder.beginRenderPass({
    colorAttachments: [
    {
      view: renderTarget.createView(),
      loadOp: 'load',
      storeOp: 'store'
    }]

  });
  for (let i = 0; i < 4; ++i) {
    renderPassEncoder.setBindGroup(i, bindGroups[i]);
  }
  renderPassEncoder.setPipeline(renderPipeline);
  renderPassEncoder.draw(1);
  renderPassEncoder.end();

  t.queue.submit([commandEncoder.finish()]);

  t.expectSingleColor(renderTarget, format, {
    size: [1, 1, 1],
    exp: { R: outputColor[0], G: outputColor[1], B: outputColor[2], A: outputColor[3] }
  });
});

g.test('pipeline_layout_with_null_bind_group_layout,compute').
desc(
  `
Tests that using a compute pipeline created with a pipeline layout that has null bind group layout
works correctly.
`
).
params((u) =>
u.
combine('emptyBindGroupLayoutType', ['Null', 'Undefined', 'Empty']).
combine('emptyBindGroupLayoutIndex', [0, 1, 2, 3])
).
fn((t) => {
  const { emptyBindGroupLayoutType, emptyBindGroupLayoutIndex } = t.params;

  let declarations = '';
  let statement = 'output = 0u ';

  const outputBuffer = t.createBufferTracked({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  });
  let expectedValue = 0;

  const bindGroupLayouts = [];
  const bindGroups = [];
  let outputDeclared = false;
  for (let bindGroupIndex = 0; bindGroupIndex < 4; ++bindGroupIndex) {
    const inputBuffer = t.makeBufferWithContents(
      new Uint32Array([bindGroupIndex + 1]),
      GPUBufferUsage.UNIFORM
    );

    const bindGroupLayoutEntries = [];
    const bindGroupEntries = [];
    bindGroupLayoutEntries.push({
      binding: 0,
      visibility: GPUConst.ShaderStage.COMPUTE,
      buffer: {
        type: 'uniform',
        minBindingSize: 4
      }
    });
    bindGroupEntries.push({
      binding: 0,
      resource: {
        buffer: inputBuffer
      }
    });

    // Set `null`, `undefined` or empty bind group layout in `bindGroupLayouts` which is used in
    // the creation of pipeline layout
    if (bindGroupIndex === emptyBindGroupLayoutIndex) {
      switch (emptyBindGroupLayoutType) {
        case 'Null':
          bindGroupLayouts.push(null);
          break;
        case 'Undefined':
          bindGroupLayouts.push(undefined);
          break;
        case 'Empty':
          bindGroupLayouts.push(
            t.device.createBindGroupLayout({
              entries: []
            })
          );
          break;
      }

      // Still create and set the bind group when the corresponding bind group layout in the
      // compute pipeline is null. The value in the output buffer should not be affected by the
      // buffer in this bind group
      const bindGroup = t.device.createBindGroup({
        layout: t.device.createBindGroupLayout({
          entries: bindGroupLayoutEntries
        }),
        entries: bindGroupEntries
      });
      bindGroups.push(bindGroup);
      continue;
    }

    declarations += `@group(${bindGroupIndex}) @binding(0) var<uniform> input${bindGroupIndex} : u32;\n`;
    statement += ` + input${bindGroupIndex}`;

    // Set the output storage buffer
    if (!outputDeclared) {
      bindGroupLayoutEntries.push({
        binding: 1,
        visibility: GPUConst.ShaderStage.COMPUTE,
        buffer: {
          type: 'storage',
          minBindingSize: 4
        }
      });
      bindGroupEntries.push({
        binding: 1,
        resource: {
          buffer: outputBuffer
        }
      });
      declarations += `@group(${bindGroupIndex}) @binding(1) var<storage, read_write> output : u32;\n`;
      outputDeclared = true;
    }

    // Set the input uniform buffers
    const bindGroupLayout = t.device.createBindGroupLayout({
      entries: bindGroupLayoutEntries
    });
    bindGroupLayouts.push(bindGroupLayout);

    const bindGroup = t.device.createBindGroup({
      layout: bindGroupLayout,
      entries: bindGroupEntries
    });
    bindGroups.push(bindGroup);

    // Compute the expected output value in the output storage buffer
    expectedValue += bindGroupIndex + 1;
  }

  const pipelineLayout = t.device.createPipelineLayout({
    bindGroupLayouts
  });

  const code = `
    ${declarations}
    @compute @workgroup_size(1)
    fn main() {
      ${statement};
    }
    `;
  const module = t.device.createShaderModule({
    code
  });
  const computePipeline = t.device.createComputePipeline({
    layout: pipelineLayout,
    compute: {
      module
    }
  });

  const commandEncoder = t.device.createCommandEncoder();
  const computePassEncoder = commandEncoder.beginComputePass();
  for (let i = 0; i < bindGroups.length; ++i) {
    computePassEncoder.setBindGroup(i, bindGroups[i]);
  }
  computePassEncoder.setPipeline(computePipeline);
  computePassEncoder.dispatchWorkgroups(1);
  computePassEncoder.end();

  t.queue.submit([commandEncoder.finish()]);

  const expectedValues = new Uint32Array([expectedValue]);
  t.expectGPUBufferValuesEqual(outputBuffer, expectedValues);
});