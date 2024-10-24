/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests limitations of createShaderModule in compat mode.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../common/util/data_tables.js';
import { kCompatModeUnsupportedStorageTextureFormats } from '../../../../format_info.js';
import { CompatibilityTest } from '../../../compatibility_test.js';

export const g = makeTestGroup(CompatibilityTest);

g.test('sample_mask').
desc(
  `
Tests that you can not create a render pipeline that uses sample_mask in compat mode.

- Test that a pipeline with a shader that uses sample_mask fails.
- Test that a pipeline that references a module that has a shader that uses sample_mask
  but the pipeline does not reference that shader succeeds.
    `
).
params((u) =>
u.
combine('entryPoint', ['fsWithoutSampleMaskUsage', 'fsWithSampleMaskUsage']).
combine('async', [false, true])
).
fn((t) => {
  const { entryPoint, async } = t.params;
  const module = t.device.createShaderModule({
    code: `
        @vertex fn vs() -> @builtin(position) vec4f {
            return vec4f(1);
        }
        struct Output {
            @builtin(sample_mask) mask_out: u32,
            @location(0) color : vec4f,
        }
        @fragment fn fsWithoutSampleMaskUsage() -> @location(0) vec4f {
            return vec4f(1.0, 1.0, 1.0, 1.0);
        }
        @fragment fn fsWithSampleMaskUsage() -> Output {
            var o: Output;
            // We need to make sure this sample_mask isn't optimized out even if its value equals "no op".
            o.mask_out = 0xFFFFFFFFu;
            o.color = vec4f(1.0, 1.0, 1.0, 1.0);
            return o;
        }
      `
  });
  const isValid = !t.isCompatibility || entryPoint === 'fsWithoutSampleMaskUsage';
  t.doCreateRenderPipelineTest(async, isValid, {
    layout: 'auto',
    vertex: { module },
    fragment: {
      module,
      entryPoint,
      targets: [{ format: 'rgba8unorm' }]
    }
  });
});

g.test('sample_index').
desc(
  `
Tests that you can not create a render pipeline that uses sample_index in compat mode.

- Test that a pipeline with a shader that uses sample_index fails.
- Test that a pipeline that references a module that has a shader that uses sample_index
  but the pipeline does not reference that shader succeeds.
    `
).
params((u) =>
u.
combine('entryPoint', ['fsWithoutSampleIndexUsage', 'fsWithSampleIndexUsage']).
combine('async', [false, true])
).
fn((t) => {
  const { entryPoint, async } = t.params;

  const module = t.device.createShaderModule({
    code: `
        @vertex fn vs() -> @builtin(position) vec4f {
            return vec4f(1);
        }
        @fragment fn fsWithoutSampleIndexUsage() -> @location(0) vec4f {
            return vec4f(0);
        }
        @fragment fn fsWithSampleIndexUsage(@builtin(sample_index) sampleIndex: u32) -> @location(0) vec4f {
            _ = sampleIndex;
            return vec4f(0);
        }
          `
  });

  const isValid = !t.isCompatibility || entryPoint === 'fsWithoutSampleIndexUsage';
  t.doCreateRenderPipelineTest(async, isValid, {
    layout: 'auto',
    vertex: { module },
    fragment: {
      module,
      entryPoint,
      targets: [{ format: 'rgba8unorm' }]
    }
  });
});

g.test('interpolate').
desc(
  `Tests that you can not create a render pipeline that uses interpolate(linear), interpolate(...,sample),
     interpolate(flat), nor interpolate(flat, first) in compat mode.`
).
params((u) =>
u.
combineWithParams([
{ success: false, interpolate: '@interpolate(linear)' },
{ success: false, interpolate: '@interpolate(linear, sample)' },
{ success: false, interpolate: '@interpolate(perspective, sample)' },
{ success: false, interpolate: '@interpolate(flat)' },
{ success: false, interpolate: '@interpolate(flat, first)' },
{ success: true, interpolate: '@interpolate(flat, either)' }]
).
combine('entryPoint', [
'fsWithoutInterpolationUsage',
'fsWithInterpolationUsage1',
'fsWithInterpolationUsage2',
'fsWithInterpolationUsage3']
).
combine('async', [false, true])
).
fn((t) => {
  const { interpolate, success, entryPoint, async } = t.params;

  const module = t.device.createShaderModule({
    code: `
        struct Vertex {
            @builtin(position) pos: vec4f,
            @location(0) ${interpolate} color : vec4f,
        };
        @vertex fn vs() -> Vertex {
            var v: Vertex;
            v.pos = vec4f(1);
            v.color = vec4f(1);
            return v;
        }
        @fragment fn fsWithoutInterpolationUsage() -> @location(0) vec4f {
            return vec4f(1);
        }
        @fragment fn fsWithInterpolationUsage1(v: Vertex) -> @location(0) vec4f {
            return vec4f(1);
        }
        @fragment fn fsWithInterpolationUsage2(v: Vertex) -> @location(0) vec4f {
            return v.pos;
        }
        @fragment fn fsWithInterpolationUsage3(v: Vertex) -> @location(0) vec4f {
            return v.color;
        }
        `
  });

  const isValid = success || !t.isCompatibility || entryPoint === 'fsWithoutInterpolationUsage';
  t.doCreateRenderPipelineTest(async, isValid, {
    layout: 'auto',
    vertex: { module },
    fragment: {
      entryPoint,
      module,
      targets: [{ format: 'rgba8unorm' }]
    }
  });
});

g.test('unsupportedStorageTextureFormats,computePipeline').
desc(
  `Tests that you can not create a compute pipeline that uses an
     unsupported storage texture format in compat mode.`
).
params((u) =>
u.
combine('format', kCompatModeUnsupportedStorageTextureFormats).
combine('entryPoint', ['csWithoutStorageUsage', 'csWithStorageUsage']).
combine('async', [false, true])
).
fn((t) => {
  const { format, entryPoint, async } = t.params;

  const module = t.device.createShaderModule({
    code: `
        @group(0) @binding(0) var s: texture_storage_2d<${format}, read>;
        @compute @workgroup_size(1) fn csWithoutStorageUsage() {
        }
        @compute @workgroup_size(1) fn csWithStorageUsage() {
            _ = textureLoad(s, vec2u(0));
        }
      `
  });

  const isValid = !t.isCompatibility || entryPoint === 'csWithoutStorageUsage';
  t.doCreateComputePipelineTest(async, isValid, {
    layout: 'auto',
    compute: { module, entryPoint }
  });
});

g.test('unsupportedStorageTextureFormats,renderPipeline').
desc(
  `Tests that you can not create a render pipeline that uses an
     unsupported storage texture format in compat mode.`
).
params((u) =>
u.
combine('format', kCompatModeUnsupportedStorageTextureFormats).
combine('entryPoint', ['vsWithoutStorageUsage', 'vsWithStorageUsage']).
combine('async', [false, true])
).
fn((t) => {
  const { format, entryPoint, async } = t.params;

  const module = t.device.createShaderModule({
    code: `
        @group(0) @binding(0) var s: texture_storage_2d<${format}, read>;
        @vertex fn vsWithoutStorageUsage() -> @builtin(position) vec4f {
            return vec4f(0);
        }
        @vertex fn vsWithStorageUsage() -> @builtin(position) vec4f {
            _ = textureLoad(s, vec2u(0));
            return vec4f(0);
        }
      `
  });

  const isValid = !t.isCompatibility || entryPoint === 'vsWithoutStorageUsage';
  t.doCreateRenderPipelineTest(async, isValid, {
    layout: 'auto',
    vertex: { module, entryPoint },
    depthStencil: { format: 'depth32float', depthWriteEnabled: true, depthCompare: 'always' }
  });
});

const kDepthTextureTypeToParams = {
  texture_depth_2d: 'vec2u(0), 0',
  texture_depth_2d_array: 'vec2u(0), 0, 0',
  texture_depth_multisampled_2d: 'vec2u(0), 0'
};
const kDepthTextureTypes = keysOf(kDepthTextureTypeToParams);

g.test('textureLoad_with_depth_textures,computePipeline').
desc(
  `Tests that you can not create a compute pipeline that uses textureLoad with a depth texture in compat mode.`
).
params((u) =>
u.
combine('type', kDepthTextureTypes).
combine('entryPoint', ['csWithoutDepthUsage', 'csWithDepthUsage']).
combine('async', [false, true])
).
fn((t) => {
  const { type, entryPoint, async } = t.params;
  const params = kDepthTextureTypeToParams[type];

  const module = t.device.createShaderModule({
    code: `
        @group(0) @binding(0) var t: ${type};
        @compute @workgroup_size(1) fn csWithoutDepthUsage() {
        }
        @compute @workgroup_size(1) fn csWithDepthUsage() {
            _ = textureLoad(t, ${params});
        }
      `
  });

  const isValid = !t.isCompatibility || entryPoint === 'csWithoutDepthUsage';
  t.doCreateComputePipelineTest(async, isValid, {
    layout: 'auto',
    compute: { module, entryPoint }
  });
});

g.test('textureLoad_with_depth_textures,renderPipeline').
desc(
  `Tests that you can not create a render pipeline that uses textureLoad with a depth texture in compat mode.`
).
params((u) =>
u.
combine('type', kDepthTextureTypes).
combine('entryPoint', ['vsWithoutDepthUsage', 'vsWithDepthUsage']).
combine('async', [false, true])
).
fn((t) => {
  const { type, entryPoint, async } = t.params;
  const params = kDepthTextureTypeToParams[type];

  const module = t.device.createShaderModule({
    code: `
        @group(0) @binding(0) var t: ${type};
        @vertex fn vsWithoutDepthUsage() -> @builtin(position) vec4f {
            return vec4f(0);
        }
        @vertex fn vsWithDepthUsage() -> @builtin(position) vec4f {
            _ = textureLoad(t, ${params});
            return vec4f(0);
        }
      `
  });

  const isValid = !t.isCompatibility || entryPoint === 'vsWithoutDepthUsage';
  t.doCreateRenderPipelineTest(async, isValid, {
    layout: 'auto',
    vertex: { module, entryPoint },
    depthStencil: { format: 'depth32float', depthWriteEnabled: true, depthCompare: 'always' }
  });
});