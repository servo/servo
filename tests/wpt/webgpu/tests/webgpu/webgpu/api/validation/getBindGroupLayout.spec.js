/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
  getBindGroupLayout validation tests.
`;import { AllFeaturesMaxLimitsGPUTest } from '../.././gpu_test.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert } from '../../../common/util/util.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('index_range,explicit_layout').
desc(
  `
  Test that a validation error is generated if the index is greater than the maximum number of bind
  groups.
  `
).
params((u) => u.combine('index', [0, 1, 2, 3, 4, 5])).
fn((t) => {
  const { index } = t.params;

  const pipelineBindGroupLayouts = t.device.createBindGroupLayout({
    entries: []
  });

  const pipelineLayout = t.device.createPipelineLayout({
    bindGroupLayouts: [pipelineBindGroupLayouts]
  });

  const pipeline = t.device.createRenderPipeline({
    layout: pipelineLayout,
    vertex: {
      module: t.device.createShaderModule({
        code: `
            @vertex
            fn main()-> @builtin(position) vec4<f32> {
              return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
            @fragment
            fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm' }]
    }
  });

  const shouldError = index >= t.device.limits.maxBindGroups;

  t.expectValidationError(() => {
    pipeline.getBindGroupLayout(index);
  }, shouldError);
});

g.test('index_range,auto_layout').
desc(
  `
  Test that a validation error is generated if the index is greater than the maximum number of bind
  groups.
  `
).
params((u) => u.combine('index', [0, 1, 2, 3, 4, 5])).
fn((t) => {
  const { index } = t.params;

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: `
            @vertex
            fn main()-> @builtin(position) vec4<f32> {
              return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
            @group(0) @binding(0) var<uniform> binding: f32;
            @fragment
            fn main() -> @location(0) vec4<f32> {
              _ = binding;
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm' }]
    }
  });

  const shouldError = index >= t.device.limits.maxBindGroups;

  t.expectValidationError(() => {
    pipeline.getBindGroupLayout(index);
  }, shouldError);
});

g.test('unique_js_object,auto_layout').
desc(
  `
  Test that getBindGroupLayout returns a new JavaScript object for each call.
  `
).
fn((t) => {
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: `
            @vertex
            fn main()-> @builtin(position) vec4<f32> {
              return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
            @group(0) @binding(0) var<uniform> binding: f32;
            @fragment
            fn main() -> @location(0) vec4<f32> {
              _ = binding;
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm' }]
    }
  });

  const kIndex = 0;
  const bgl1 = pipeline.getBindGroupLayout(kIndex);
  bgl1.extra = 42;
  const bgl2 = pipeline.getBindGroupLayout(kIndex);

  assert(bgl1 !== bgl2, 'objects are not the same object');
  assert(bgl2.extra === undefined, 'objects do not retain expando properties');
});

g.test('unique_js_object,explicit_layout').
desc(
  `
  Test that getBindGroupLayout returns a new JavaScript object for each call.
  `
).
fn((t) => {
  const pipelineBindGroupLayouts = t.device.createBindGroupLayout({
    entries: []
  });

  const pipelineLayout = t.device.createPipelineLayout({
    bindGroupLayouts: [pipelineBindGroupLayouts]
  });

  const pipeline = t.device.createRenderPipeline({
    layout: pipelineLayout,
    vertex: {
      module: t.device.createShaderModule({
        code: `
            @vertex
            fn main()-> @builtin(position) vec4<f32> {
              return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
            @fragment
            fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm' }]
    }
  });

  const kIndex = 0;
  const bgl1 = pipeline.getBindGroupLayout(kIndex);
  bgl1.extra = 42;
  const bgl2 = pipeline.getBindGroupLayout(kIndex);

  assert(bgl1 !== bgl2, 'objects are not the same object');
  assert(bgl2.extra === undefined, 'objects do not retain expando properties');
});