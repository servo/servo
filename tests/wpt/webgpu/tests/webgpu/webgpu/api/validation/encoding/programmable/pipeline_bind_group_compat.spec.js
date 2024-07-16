/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
TODO:
- test compatibility between bind groups and pipelines
    - the binding resource in bindGroups[i].layout is "group-equivalent" (value-equal) to pipelineLayout.bgls[i].
    - in the test fn, test once without the dispatch/draw (should always be valid) and once with
      the dispatch/draw, to make sure the validation happens in dispatch/draw.
    - x= {dispatch, all draws} (dispatch/draw should be size 0 to make sure validation still happens if no-op)
    - x= all relevant stages

TODO: subsume existing test, rewrite fixture as needed.
TODO: Add externalTexture to kResourceTypes [1]
`;import { kUnitCaseParamsBuilder } from '../../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { memcpy, unreachable } from '../../../../../common/util/util.js';
import {
  kSamplerBindingTypes,
  kShaderStageCombinations,
  kBufferBindingTypes } from

'../../../../capability_info.js';
import { GPUConst } from '../../../../constants.js';
import {

  kProgrammableEncoderTypes } from
'../../../../util/command_buffer_maker.js';
import { ValidationTest } from '../../validation_test.js';

const kComputeCmds = ['dispatch', 'dispatchIndirect'];

const kRenderCmds = ['draw', 'drawIndexed', 'drawIndirect', 'drawIndexedIndirect'];


const kPipelineTypes = ['auto0', 'explicit'];

const kBindingTypes = ['auto0', 'auto1', 'explicit'];


const kEmptyBindGroup0Ndx = 0;
const kEmptyBindGroup1Ndx = 1;
const kNonEmptyBindGroup0Ndx = 2;
const kNonEmptyBindGroup1Ndx = 3;

// Swaps 2 array elements in place.
function swapArrayElements(array, ndx1, ndx2) {
  const t = array[ndx1];
  array[ndx1] = array[ndx2];
  array[ndx2] = t;
}

// Test resource type compatibility in pipeline and bind group
// [1]: Need to add externalTexture
const kResourceTypes = [
'uniformBuf',
'filtSamp',
'sampledTex',
'readonlyStorageTex',
'writeonlyStorageTex',
'readwriteStorageTex'];


function getTestCmds(
encoderType)
{
  return encoderType === 'compute pass' ? kComputeCmds : kRenderCmds;
}

const kCompatTestParams = kUnitCaseParamsBuilder.
combine('encoderType', kProgrammableEncoderTypes).
expand('call', (p) => getTestCmds(p.encoderType)).
combine('callWithZero', [true, false]);

class F extends ValidationTest {
  getIndexBuffer() {
    return this.createBufferTracked({
      size: 8 * Uint32Array.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.INDEX
    });
  }

  getIndirectBuffer(indirectParams) {
    const buffer = this.createBufferTracked({
      mappedAtCreation: true,
      size: indirectParams.length * Uint32Array.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.INDIRECT | GPUBufferUsage.COPY_DST
    });
    memcpy({ src: new Uint32Array(indirectParams) }, { dst: buffer.getMappedRange() });
    buffer.unmap();
    return buffer;
  }

  getBindingResourceType(entry) {
    if (entry.buffer !== undefined) return 'uniformBuf';
    if (entry.sampler !== undefined) return 'filtSamp';
    if (entry.texture !== undefined) return 'sampledTex';
    if (entry.storageTexture !== undefined) {
      switch (entry.storageTexture.access) {
        case undefined:
        case 'write-only':
          return 'writeonlyStorageTex';
        case 'read-only':
          return 'readonlyStorageTex';
        case 'read-write':
          return 'readwriteStorageTex';
      }
    }
    unreachable();
  }

  createRenderPipelineWithLayout(
  bindGroups)
  {
    const shader = `
      @vertex fn vs_main() -> @builtin(position) vec4<f32> {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0);
      }

      @fragment fn fs_main() -> @location(0) vec4<f32> {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
      }
    `;
    const module = this.device.createShaderModule({ code: shader });
    const pipeline = this.device.createRenderPipeline({
      layout: this.device.createPipelineLayout({
        bindGroupLayouts: bindGroups.map((entries) => this.device.createBindGroupLayout({ entries }))
      }),
      vertex: {
        module,
        entryPoint: 'vs_main'
      },
      fragment: {
        module,
        entryPoint: 'fs_main',
        targets: [{ format: 'rgba8unorm' }]
      },
      primitive: { topology: 'triangle-list' }
    });
    return pipeline;
  }

  createComputePipelineWithLayout(
  bindGroups)
  {
    const shader = `
      @compute @workgroup_size(1)
        fn main(@builtin(global_invocation_id) GlobalInvocationID : vec3<u32>) {
      }
    `;

    const module = this.device.createShaderModule({ code: shader });
    const pipeline = this.device.createComputePipeline({
      layout: this.device.createPipelineLayout({
        bindGroupLayouts: bindGroups.map((entries) => this.device.createBindGroupLayout({ entries }))
      }),
      compute: {
        module,
        entryPoint: 'main'
      }
    });
    return pipeline;
  }

  createBindGroupWithLayout(bglEntries) {
    const bgEntries = [];
    for (const entry of bglEntries) {
      const resource = this.getBindingResource(this.getBindingResourceType(entry));
      bgEntries.push({
        binding: entry.binding,
        resource
      });
    }

    return this.device.createBindGroup({
      entries: bgEntries,
      layout: this.device.createBindGroupLayout({ entries: bglEntries })
    });
  }

  doCompute(pass, call, callWithZero) {
    const x = callWithZero ? 0 : 1;
    switch (call) {
      case 'dispatch':
        pass.dispatchWorkgroups(x, 1, 1);
        break;
      case 'dispatchIndirect':
        pass.dispatchWorkgroupsIndirect(this.getIndirectBuffer([x, 1, 1]), 0);
        break;
      default:
        break;
    }
  }

  doRender(
  pass,
  call,
  callWithZero)
  {
    const vertexCount = callWithZero ? 0 : 3;
    switch (call) {
      case 'draw':
        pass.draw(vertexCount, 1, 0, 0);
        break;
      case 'drawIndexed':
        pass.setIndexBuffer(this.getIndexBuffer(), 'uint32');
        pass.drawIndexed(vertexCount, 1, 0, 0, 0);
        break;
      case 'drawIndirect':
        pass.drawIndirect(this.getIndirectBuffer([vertexCount, 1, 0, 0, 0]), 0);
        break;
      case 'drawIndexedIndirect':
        pass.setIndexBuffer(this.getIndexBuffer(), 'uint32');
        pass.drawIndexedIndirect(this.getIndirectBuffer([vertexCount, 1, 0, 0, 0]), 0);
        break;
      default:
        break;
    }
  }

  createBindGroupLayoutEntry(
  encoderType,
  resourceType,
  useU32Array)
  {
    const entry = {
      binding: 0,
      visibility: encoderType === 'compute pass' ? GPUShaderStage.COMPUTE : GPUShaderStage.FRAGMENT
    };

    switch (resourceType) {
      case 'uniformBuf':
        entry.buffer = { hasDynamicOffset: useU32Array }; // default type: uniform
        break;
      case 'filtSamp':
        entry.sampler = {}; // default type: filtering
        break;
      case 'sampledTex':
        entry.texture = {}; // default sampleType: float
        break;
      case 'readonlyStorageTex':
        entry.storageTexture = { access: 'read-only', format: 'r32float' };
        break;
      case 'writeonlyStorageTex':
        entry.storageTexture = { access: 'write-only', format: 'r32float' };
        break;
      case 'readwriteStorageTex':
        entry.storageTexture = { access: 'read-write', format: 'r32float' };
        break;
    }

    return entry;
  }

  runTest(
  encoderType,
  pipeline,
  bindGroups,
  dynamicOffsets,
  call,
  callWithZero,
  success)
  {
    const { encoder, validateFinish } = this.createEncoder(encoderType);

    if (encoder instanceof GPUComputePassEncoder) {
      encoder.setPipeline(pipeline);
    } else {
      encoder.setPipeline(pipeline);
    }

    for (let i = 0; i < bindGroups.length; i++) {
      const bindGroup = bindGroups[i];
      if (!bindGroup) {
        break;
      }
      if (dynamicOffsets) {
        encoder.setBindGroup(
          i,
          bindGroup,
          new Uint32Array(dynamicOffsets),
          0,
          dynamicOffsets.length
        );
      } else {
        encoder.setBindGroup(i, bindGroup);
      }
    }

    if (encoder instanceof GPUComputePassEncoder) {
      this.doCompute(encoder, call, callWithZero);
    } else {
      this.doRender(encoder, call, callWithZero);
    }

    validateFinish(success);
  }

  runDefaultLayoutBindingTest({
    visibility,
    empty,
    pipelineType,
    bindingType,
    swap,
    success,
    makePipelinesFn,
    doCommandFn















  }) {
    const { device } = this;
    const explicitEmptyBindGroupLayout = device.createBindGroupLayout({
      entries: []
    });
    const explicitBindGroupLayout = device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility,
        buffer: {}
      }]

    });
    const explicitPipelineLayout = device.createPipelineLayout({
      bindGroupLayouts: [
      explicitEmptyBindGroupLayout,
      explicitEmptyBindGroupLayout,
      explicitBindGroupLayout,
      explicitBindGroupLayout]

    });

    const [pipelineAuto0, pipelineAuto1, pipelineExplicit] = makePipelinesFn(
      this,
      explicitPipelineLayout
    );

    const buffer = this.createBufferTracked({
      size: 16,
      usage: GPUBufferUsage.UNIFORM
    });

    let emptyBindGroupLayouts;
    let nonEmptyBindGroupLayouts;
    const pipeline = pipelineType === 'auto0' ? pipelineAuto0 : pipelineExplicit;

    // Gets a bindGroupLayout either from the explicit layout passed in
    // or one of the 2 `layout: 'auto'` pipelines.
    const getBindGroupLayout = (
    explicitBindGroupLayout,
    bindGroupIndex) =>

    bindingType === 'explicit' ?
    explicitBindGroupLayout :
    bindingType === 'auto0' ?
    pipelineAuto0.getBindGroupLayout(bindGroupIndex) :
    pipelineAuto1.getBindGroupLayout(bindGroupIndex);

    if (empty) {
      // Testing empty:
      // - emptyBindGroupLayout comes from a possibly incompatible place.
      // - nonEmptyBindGroupLayout comes from the pipeline we'll render/compute with.
      emptyBindGroupLayouts = [
      getBindGroupLayout(explicitEmptyBindGroupLayout, kEmptyBindGroup0Ndx),
      getBindGroupLayout(explicitEmptyBindGroupLayout, kEmptyBindGroup1Ndx)];

      if (swap) {
        swapArrayElements(emptyBindGroupLayouts, 0, 1);
      }
      nonEmptyBindGroupLayouts = [
      pipeline.getBindGroupLayout(kNonEmptyBindGroup0Ndx),
      pipeline.getBindGroupLayout(kNonEmptyBindGroup1Ndx)];

    } else {
      // Testing non-empty:
      // - nonEmptyBindGroupLayout comes from a possibly incompatible place.
      // - emptyBindGroupLayout comes from the pipeline we'll render/compute with.
      nonEmptyBindGroupLayouts = [
      getBindGroupLayout(explicitBindGroupLayout, kNonEmptyBindGroup0Ndx),
      getBindGroupLayout(explicitBindGroupLayout, kNonEmptyBindGroup1Ndx)];

      if (swap) {
        swapArrayElements(nonEmptyBindGroupLayouts, 0, 1);
      }
      emptyBindGroupLayouts = [
      pipeline.getBindGroupLayout(kEmptyBindGroup0Ndx),
      pipeline.getBindGroupLayout(kEmptyBindGroup1Ndx)];

    }

    const emptyBindGroups = emptyBindGroupLayouts.map((layout) =>
    device.createBindGroup({
      layout,
      entries: []
    })
    );

    const nonEmptyBindGroups = nonEmptyBindGroupLayouts.map((layout) =>
    device.createBindGroup({
      layout,
      entries: [{ binding: 0, resource: { buffer } }]
    })
    );

    const encoder = device.createCommandEncoder();

    doCommandFn({ t: this, encoder, pipeline, emptyBindGroups, nonEmptyBindGroups });

    this.expectValidationError(() => {
      encoder.finish();
    }, !success);
  }
}

export const g = makeTestGroup(F);

g.test('bind_groups_and_pipeline_layout_mismatch').
desc(
  `
    Tests the bind groups must match the requirements of the pipeline layout.
    - bind groups required by the pipeline layout are required.
    - bind groups unused by the pipeline layout can be set or not.
    `
).
params(
  kCompatTestParams.
  beginSubcases().
  combineWithParams([
  { setBindGroup0: true, setBindGroup1: true, setUnusedBindGroup2: true, _success: true },
  { setBindGroup0: true, setBindGroup1: true, setUnusedBindGroup2: false, _success: true },
  { setBindGroup0: true, setBindGroup1: false, setUnusedBindGroup2: true, _success: false },
  { setBindGroup0: false, setBindGroup1: true, setUnusedBindGroup2: true, _success: false },
  { setBindGroup0: false, setBindGroup1: false, setUnusedBindGroup2: false, _success: false }]
  ).
  combine('useU32Array', [false, true])
).
fn((t) => {
  const {
    encoderType,
    call,
    callWithZero,
    setBindGroup0,
    setBindGroup1,
    setUnusedBindGroup2,
    _success,
    useU32Array
  } = t.params;
  const visibility =
  encoderType === 'compute pass' ? GPUShaderStage.COMPUTE : GPUShaderStage.VERTEX;

  const bindGroupLayouts = [
  // bind group layout 0
  [
  {
    binding: 0,
    visibility,
    buffer: { hasDynamicOffset: useU32Array } // default type: uniform
  }],

  // bind group layout 1
  [
  {
    binding: 0,
    visibility,
    buffer: { hasDynamicOffset: useU32Array } // default type: uniform
  }]];



  // Create required bind groups
  const bindGroup0 = setBindGroup0 ? t.createBindGroupWithLayout(bindGroupLayouts[0]) : undefined;
  const bindGroup1 = setBindGroup1 ? t.createBindGroupWithLayout(bindGroupLayouts[1]) : undefined;
  const unusedBindGroup2 = setUnusedBindGroup2 ?
  t.createBindGroupWithLayout(bindGroupLayouts[1]) :
  undefined;

  // Create fixed pipeline
  const pipeline =
  encoderType === 'compute pass' ?
  t.createComputePipelineWithLayout(bindGroupLayouts) :
  t.createRenderPipelineWithLayout(bindGroupLayouts);

  const dynamicOffsets = useU32Array ? [0] : undefined;

  // Test without the dispatch/draw (should always be valid)
  t.runTest(
    encoderType,
    pipeline,
    [bindGroup0, bindGroup1, unusedBindGroup2],
    dynamicOffsets,
    undefined,
    false,
    true
  );

  // Test with the dispatch/draw, to make sure the validation happens in dispatch/draw.
  t.runTest(
    encoderType,
    pipeline,
    [bindGroup0, bindGroup1, unusedBindGroup2],
    dynamicOffsets,
    call,
    callWithZero,
    _success
  );
});

g.test('buffer_binding,render_pipeline').
desc(
  `
  The GPUBufferBindingLayout bindings configure should be exactly
  same in PipelineLayout and bindgroup.
  - TODO: test more draw functions, e.g. indirect
  - TODO: test more visibilities, e.g. vertex
  - TODO: bind group should be created with different layout
  `
).
params((u) => u.combine('type', kBufferBindingTypes)).
fn((t) => {
  const { type } = t.params;

  // Create fixed bindGroup
  const uniformBuffer = t.getUniformBuffer();

  const bindGroup = t.device.createBindGroup({
    entries: [
    {
      binding: 0,
      resource: {
        buffer: uniformBuffer
      }
    }],

    layout: t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.FRAGMENT,
        buffer: {} // default type: uniform
      }]

    })
  });

  // Create pipeline with different layouts
  const pipeline = t.createRenderPipelineWithLayout([
  [
  {
    binding: 0,
    visibility: GPUShaderStage.FRAGMENT,
    buffer: {
      type
    }
  }]]

  );

  const { encoder, validateFinish } = t.createEncoder('render pass');
  encoder.setPipeline(pipeline);
  encoder.setBindGroup(0, bindGroup);
  encoder.draw(3);

  validateFinish(type === undefined || type === 'uniform');
});

g.test('sampler_binding,render_pipeline').
desc(
  `
  The GPUSamplerBindingLayout bindings configure should be exactly
  same in PipelineLayout and bindgroup.
  - TODO: test more draw functions, e.g. indirect
  - TODO: test more visibilities, e.g. vertex
  `
).
params((u) =>
u //
.combine('bglType', kSamplerBindingTypes).
combine('bgType', kSamplerBindingTypes)
).
fn((t) => {
  const { bglType, bgType } = t.params;
  const bindGroup = t.device.createBindGroup({
    entries: [
    {
      binding: 0,
      resource:
      bgType === 'comparison' ?
      t.device.createSampler({ compare: 'always' }) :
      t.device.createSampler()
    }],

    layout: t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.FRAGMENT,
        sampler: { type: bgType }
      }]

    })
  });

  // Create pipeline with different layouts
  const pipeline = t.createRenderPipelineWithLayout([
  [
  {
    binding: 0,
    visibility: GPUShaderStage.FRAGMENT,
    sampler: {
      type: bglType
    }
  }]]

  );

  const { encoder, validateFinish } = t.createEncoder('render pass');
  encoder.setPipeline(pipeline);
  encoder.setBindGroup(0, bindGroup);
  encoder.draw(3);

  validateFinish(bglType === bgType);
});

g.test('bgl_binding_mismatch').
desc(
  'Tests the binding number must exist or not exist in both bindGroups[i].layout and pipelineLayout.bgls[i]'
).
params(
  kCompatTestParams.
  beginSubcases().
  combineWithParams([
  { bgBindings: [0, 1, 2], plBindings: [0, 1, 2], _success: true },
  { bgBindings: [0, 1, 2], plBindings: [0, 1, 3], _success: false },
  { bgBindings: [0, 2], plBindings: [0, 2], _success: true },
  { bgBindings: [0, 2], plBindings: [2, 0], _success: true },
  { bgBindings: [0, 1, 2], plBindings: [0, 1], _success: false },
  { bgBindings: [0, 1], plBindings: [0, 1, 2], _success: false }]
  ).
  combine('useU32Array', [false, true])
).
fn((t) => {
  const { encoderType, call, callWithZero, bgBindings, plBindings, _success, useU32Array } =
  t.params;
  const visibility =
  encoderType === 'compute pass' ? GPUShaderStage.COMPUTE : GPUShaderStage.VERTEX;

  const bglEntries = [];
  for (const binding of bgBindings) {
    bglEntries.push({
      binding,
      visibility,
      buffer: { hasDynamicOffset: useU32Array } // default type: uniform
    });
  }
  const bindGroup = t.createBindGroupWithLayout(bglEntries);

  const plEntries = [[]];
  for (const binding of plBindings) {
    plEntries[0].push({
      binding,
      visibility,
      buffer: { hasDynamicOffset: useU32Array } // default type: uniform
    });
  }
  const pipeline =
  encoderType === 'compute pass' ?
  t.createComputePipelineWithLayout(plEntries) :
  t.createRenderPipelineWithLayout(plEntries);

  const dynamicOffsets = useU32Array ? new Array(bgBindings.length).fill(0) : undefined;

  // Test without the dispatch/draw (should always be valid)
  t.runTest(encoderType, pipeline, [bindGroup], dynamicOffsets, undefined, false, true);

  // Test with the dispatch/draw, to make sure the validation happens in dispatch/draw.
  t.runTest(encoderType, pipeline, [bindGroup], dynamicOffsets, call, callWithZero, _success);
});

g.test('bgl_visibility_mismatch').
desc('Tests the visibility in bindGroups[i].layout and pipelineLayout.bgls[i] must be matched').
params(
  kCompatTestParams.
  beginSubcases().
  combine('bgVisibility', kShaderStageCombinations).
  expand('plVisibility', (p) =>
  p.encoderType === 'compute pass' ?
  [GPUConst.ShaderStage.COMPUTE] :
  [
  GPUConst.ShaderStage.VERTEX,
  GPUConst.ShaderStage.FRAGMENT,
  GPUConst.ShaderStage.VERTEX | GPUConst.ShaderStage.FRAGMENT]

  ).
  combine('useU32Array', [false, true])
).
fn((t) => {
  const { encoderType, call, callWithZero, bgVisibility, plVisibility, useU32Array } = t.params;

  const bglEntries = [
  {
    binding: 0,
    visibility: bgVisibility,
    buffer: { hasDynamicOffset: useU32Array } // default type: uniform
  }];

  const bindGroup = t.createBindGroupWithLayout(bglEntries);

  const plEntries = [
  [
  {
    binding: 0,
    visibility: plVisibility,
    buffer: { hasDynamicOffset: useU32Array } // default type: uniform
  }]];


  const pipeline =
  encoderType === 'compute pass' ?
  t.createComputePipelineWithLayout(plEntries) :
  t.createRenderPipelineWithLayout(plEntries);

  const dynamicOffsets = useU32Array ? [0] : undefined;

  // Test without the dispatch/draw (should always be valid)
  t.runTest(encoderType, pipeline, [bindGroup], dynamicOffsets, undefined, false, true);

  // Test with the dispatch/draw, to make sure the validation happens in dispatch/draw.
  t.runTest(
    encoderType,
    pipeline,
    [bindGroup],
    dynamicOffsets,
    call,
    callWithZero,
    bgVisibility === plVisibility
  );
});

g.test('bgl_resource_type_mismatch').
desc(
  `
  Tests the binding resource type in bindGroups[i].layout and pipelineLayout.bgls[i] must be matched
  - TODO: Test externalTexture
  `
).
params(
  kCompatTestParams.
  beginSubcases().
  combine('bgResourceType', kResourceTypes).
  combine('plResourceType', kResourceTypes).
  expand('useU32Array', (p) => p.bgResourceType === 'uniformBuf' ? [true, false] : [false])
).
fn((t) => {
  const { encoderType, call, callWithZero, bgResourceType, plResourceType, useU32Array } =
  t.params;

  const bglEntries = [
  t.createBindGroupLayoutEntry(encoderType, bgResourceType, useU32Array)];

  const bindGroup = t.createBindGroupWithLayout(bglEntries);

  const plEntries = [
  [t.createBindGroupLayoutEntry(encoderType, plResourceType, useU32Array)]];

  const pipeline =
  encoderType === 'compute pass' ?
  t.createComputePipelineWithLayout(plEntries) :
  t.createRenderPipelineWithLayout(plEntries);

  const dynamicOffsets = useU32Array ? [0] : undefined;

  // Test without the dispatch/draw (should always be valid)
  t.runTest(encoderType, pipeline, [bindGroup], dynamicOffsets, undefined, false, true);

  // Test with the dispatch/draw, to make sure the validation happens in dispatch/draw.
  t.runTest(
    encoderType,
    pipeline,
    [bindGroup],
    dynamicOffsets,
    call,
    callWithZero,
    bgResourceType === plResourceType
  );
});

g.test('empty_bind_group_layouts_requires_empty_bind_groups,compute_pass').
desc(
  `
  Test that a compute pipeline with empty bind groups layouts requires empty bind groups to be set.
  `
).
params((u) =>
u.
combine('bindGroupLayoutEntryCount', [3, 4]).
combine('computeCommand', ['dispatchIndirect', 'dispatch'])
).
fn((t) => {
  const { bindGroupLayoutEntryCount, computeCommand } = t.params;

  const emptyBGLCount = 4;
  const emptyBGL = t.device.createBindGroupLayout({ entries: [] });
  const emptyBGLs = [];
  for (let i = 0; i < emptyBGLCount; i++) {
    emptyBGLs.push(emptyBGL);
  }

  const pipelineLayout = t.device.createPipelineLayout({
    bindGroupLayouts: emptyBGLs
  });

  const pipeline = t.device.createComputePipeline({
    layout: pipelineLayout,
    compute: {
      module: t.device.createShaderModule({
        code: '@compute @workgroup_size(1) fn main() {}'
      }),
      entryPoint: 'main'
    }
  });

  const emptyBindGroup = t.device.createBindGroup({
    layout: emptyBGL,
    entries: []
  });

  const encoder = t.device.createCommandEncoder();
  const computePass = encoder.beginComputePass();
  computePass.setPipeline(pipeline);
  for (let i = 0; i < bindGroupLayoutEntryCount; i++) {
    computePass.setBindGroup(i, emptyBindGroup);
  }

  t.doCompute(computePass, computeCommand, true);
  computePass.end();

  const success = bindGroupLayoutEntryCount === emptyBGLCount;

  t.expectValidationError(() => {
    encoder.finish();
  }, !success);
});

g.test('empty_bind_group_layouts_requires_empty_bind_groups,render_pass').
desc(
  `
  Test that a render pipeline with empty bind groups layouts requires empty bind groups to be set.
  `
).
params((u) =>
u.
combine('bindGroupLayoutEntryCount', [3, 4]).
combine('renderCommand', [
'draw',
'drawIndexed',
'drawIndirect',
'drawIndexedIndirect']
)
).
fn((t) => {
  const { bindGroupLayoutEntryCount, renderCommand } = t.params;

  const emptyBGLCount = 4;
  const emptyBGL = t.device.createBindGroupLayout({ entries: [] });
  const emptyBGLs = [];
  for (let i = 0; i < emptyBGLCount; i++) {
    emptyBGLs.push(emptyBGL);
  }

  const pipelineLayout = t.device.createPipelineLayout({
    bindGroupLayouts: emptyBGLs
  });

  const colorFormat = 'rgba8unorm';
  const pipeline = t.device.createRenderPipeline({
    layout: pipelineLayout,
    vertex: {
      module: t.device.createShaderModule({
        code: `@vertex fn main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }`
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `@fragment fn main() {}`
      }),
      entryPoint: 'main',
      targets: [{ format: colorFormat, writeMask: 0 }]
    }
  });

  const emptyBindGroup = t.device.createBindGroup({
    layout: emptyBGL,
    entries: []
  });

  const encoder = t.device.createCommandEncoder();

  const attachmentTexture = t.createTextureTracked({
    format: 'rgba8unorm',
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.RENDER_ATTACHMENT
  });

  const renderPass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: attachmentTexture.createView(),
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });

  renderPass.setPipeline(pipeline);
  for (let i = 0; i < bindGroupLayoutEntryCount; i++) {
    renderPass.setBindGroup(i, emptyBindGroup);
  }
  t.doRender(renderPass, renderCommand, true);
  renderPass.end();

  const success = bindGroupLayoutEntryCount === emptyBGLCount;

  t.expectValidationError(() => {
    encoder.finish();
  }, !success);
});

// pipelineType specifies which pipeline to try to render/compute with
//    auto0 = the first `layout: 'auto'` pipeline
//    explicit = a pipeline crated with an explicit pipeline layout using explicit bind group layouts
//
// bindingType specifies where to get bindGroupLayouts to use to create bindGroups
//    auto0 = the first `layout: 'auto'` pipeline
//    auto1 = the second `layout: 'auto'` pipeline
//    explicit = a pipeline crated with an explicit pipeline layout using explicit bind group layouts
//
// swap specifies to swap the bindgroups we're testing. We test 2 of each type, 2 empty bindgroups and
// 2 non-empty bindgroups. The 2 empty bindgroups, when swapped should still be compatible. Similarly
// the 2 non-empty bindgroups, when swapped, should still be compatible.
const kPipelineTypesAndBindingTypeParams = [
{ pipelineType: 'auto0', bindingType: 'auto0', swap: false, _success: true },
{ pipelineType: 'explicit', bindingType: 'explicit', swap: false, _success: true },
{ pipelineType: 'explicit', bindingType: 'auto0', swap: false, _success: false },
{ pipelineType: 'auto0', bindingType: 'explicit', swap: false, _success: false },
{ pipelineType: 'auto0', bindingType: 'auto1', swap: false, _success: false },
{ pipelineType: 'auto0', bindingType: 'auto0', swap: true, _success: true }];


g.test('default_bind_group_layouts_never_match,compute_pass').
desc(
  `
  Test that bind groups created with default bind group layouts never match other layouts, including empty bind groups.

  * Test that a pipeline with an explicit layout can not be used with a bindGroup from an auto layout
  * Test that a pipeline with an auto layout can not be used with a bindGroup from an explicit layout
  * Test that an auto layout from one pipeline can not be used with an auto layout from a different pipeline.
  * Test matching bindgroup layouts on the same default layout pipeline are compatible. In other words if
    you only define group(2) then group(0)'s empty layout and group(1)'s empty layout should be compatible.
    Similarly if group(2) and group(3) have the same types of resources they should be compatible.
  `
).
params((u) =>
u.
combineWithParams(kPipelineTypesAndBindingTypeParams).
combine('empty', [false, true]).
combine('computeCommand', ['dispatchIndirect', 'dispatch'])
).
fn((t) => {
  const { pipelineType, bindingType, swap, _success: success, computeCommand, empty } = t.params;

  t.runDefaultLayoutBindingTest({
    visibility: GPUShaderStage.COMPUTE,
    empty,
    pipelineType,
    bindingType,
    swap,
    success,
    makePipelinesFn: (t, explicitPipelineLayout) => {
      return ['auto', 'auto', explicitPipelineLayout].map((layout) =>
      t.device.createComputePipeline({
        layout,
        compute: {
          module: t.device.createShaderModule({
            code: `
                @group(2) @binding(0) var<uniform> u1: vec4f;
                @group(3) @binding(0) var<uniform> u2: vec4f;
                @compute @workgroup_size(2) fn main() { _ = u1; _ = u2; }
              `
          }),
          entryPoint: 'main'
        }
      })
      );
    },
    doCommandFn: ({ t, encoder, pipeline, emptyBindGroups, nonEmptyBindGroups }) => {
      const pass = encoder.beginComputePass();
      pass.setPipeline(pipeline);
      pass.setBindGroup(kEmptyBindGroup0Ndx, emptyBindGroups[0]);
      pass.setBindGroup(kEmptyBindGroup1Ndx, emptyBindGroups[1]);
      pass.setBindGroup(kNonEmptyBindGroup0Ndx, nonEmptyBindGroups[0]);
      pass.setBindGroup(kNonEmptyBindGroup1Ndx, nonEmptyBindGroups[1]);
      t.doCompute(pass, computeCommand, true);
      pass.end();
    }
  });
});

g.test('default_bind_group_layouts_never_match,render_pass').
desc(
  `
  Test that bind groups created with default bind group layouts never match other layouts, including empty bind groups.

  * Test that a pipeline with an explicit layout can not be used with a bindGroup from an auto layout
  * Test that a pipeline with an auto layout can not be used with a bindGroup from an explicit layout
  * Test that an auto layout from one pipeline can not be used with an auto layout from a different pipeline.
  * Test matching bindgroup layouts on the same default layout pipeline are compatible. In other words if
    you only define group(2) then group(0)'s empty layout and group(1)'s empty layout should be compatible.
    Similarly if group(2) and group(3) have the same types of resources they should be compatible.
  `
).
params((u) =>
u.
combineWithParams(kPipelineTypesAndBindingTypeParams).
combine('empty', [false, true]).
combine('renderCommand', [
'draw',
'drawIndexed',
'drawIndirect',
'drawIndexedIndirect']
)
).
fn((t) => {
  const { pipelineType, bindingType, swap, _success: success, renderCommand, empty } = t.params;

  t.runDefaultLayoutBindingTest({
    visibility: GPUShaderStage.VERTEX,
    empty,
    pipelineType,
    bindingType,
    swap,
    success,
    makePipelinesFn: (t, explicitPipelineLayout) => {
      return ['auto', 'auto', explicitPipelineLayout].map(
        (layout) => {
          const colorFormat = 'rgba8unorm';
          return t.device.createRenderPipeline({
            layout,
            vertex: {
              module: t.device.createShaderModule({
                code: `
                @group(2) @binding(0) var<uniform> u1: vec4f;
                @group(3) @binding(0) var<uniform> u2: vec4f;
                @vertex fn main() -> @builtin(position) vec4f { return u1 + u2; }
              `
              }),
              entryPoint: 'main'
            },
            fragment: {
              module: t.device.createShaderModule({
                code: `@fragment fn main() {}`
              }),
              entryPoint: 'main',
              targets: [{ format: colorFormat, writeMask: 0 }]
            }
          });
        }
      );
    },
    doCommandFn: ({ t, encoder, pipeline, emptyBindGroups, nonEmptyBindGroups }) => {
      const attachmentTexture = t.createTextureTracked({
        format: 'rgba8unorm',
        size: { width: 16, height: 16, depthOrArrayLayers: 1 },
        usage: GPUTextureUsage.RENDER_ATTACHMENT
      });

      const renderPass = encoder.beginRenderPass({
        colorAttachments: [
        {
          view: attachmentTexture.createView(),
          clearValue: [0, 0, 0, 0],
          loadOp: 'clear',
          storeOp: 'store'
        }]

      });

      renderPass.setPipeline(pipeline);
      renderPass.setBindGroup(kEmptyBindGroup0Ndx, emptyBindGroups[0]);
      renderPass.setBindGroup(kEmptyBindGroup1Ndx, emptyBindGroups[1]);
      renderPass.setBindGroup(kNonEmptyBindGroup0Ndx, nonEmptyBindGroups[0]);
      renderPass.setBindGroup(kNonEmptyBindGroup1Ndx, nonEmptyBindGroups[1]);
      t.doRender(renderPass, renderCommand, true);
      renderPass.end();
    }
  });
});