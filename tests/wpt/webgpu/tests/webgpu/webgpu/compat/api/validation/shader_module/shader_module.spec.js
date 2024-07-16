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
desc(`Tests that you can not create a shader module that uses sample_mask in compat mode.`).
fn((t) => {
  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () =>
    t.device.createShaderModule({
      code: `
            @vertex fn vs() -> @builtin(position) vec4f {
                return vec4f(1);
            }
            struct Output {
                @builtin(sample_mask) mask_out: u32,
                @location(0) color : vec4f,
            }
            @fragment fn fsWithSampleMaskUsage() -> Output {
                var o: Output;
                // We need to make sure this sample_mask isn't optimized out even if its value equals "no op".
                o.mask_out = 0xFFFFFFFFu;
                o.color = vec4f(1.0, 1.0, 1.0, 1.0);
                return o;
            }
          `
    }),
    true
  );
});

g.test('sample_index').
desc(`Tests that you can not create a shader module that uses sample_index in compat mode.`).
fn((t) => {
  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () =>
    t.device.createShaderModule({
      code: `
            @vertex fn vs() -> @builtin(position) vec4f {
                return vec4f(1);
            }
            @fragment fn fsWithSampleIndexUsage(@builtin(sample_index) sampleIndex: u32) -> @location(0) vec4f {
                _ = sampleIndex;
                return vec4f(0);
            }
              `
    }),
    true
  );
});

g.test('interpolate').
desc(
  `Tests that you can not create a shader module that uses interpolate(linear), interpolate(...,sample),
     interpolate(flat), nor interpolate(flat, first) in compat mode.`
).
params((u) =>
u.combineWithParams([
{ success: true, interpolate: '' },
{ success: false, interpolate: '@interpolate(linear)' },
{ success: false, interpolate: '@interpolate(linear, sample)' },
{ success: false, interpolate: '@interpolate(perspective, sample)' },
{ success: false, interpolate: '@interpolate(flat)' },
{ success: false, interpolate: '@interpolate(flat, first)' },
{ success: true, interpolate: '@interpolate(flat, either)' }]
)
).
fn((t) => {
  const { interpolate, success } = t.params;

  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () =>
    t.device.createShaderModule({
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
            @fragment fn fsWithInterpolationUsage(v: Vertex) -> @location(0) vec4f {
                return v.color;
            }
            `
    }),
    !success
  );
});

g.test('unsupportedStorageTextureFormats').
desc(
  `Tests that you can not create a shader module with unsupported storage texture formats in compat mode.`
).
params((u) => u.combine('format', kCompatModeUnsupportedStorageTextureFormats)).
fn((t) => {
  const { format } = t.params;

  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () =>
    t.device.createShaderModule({
      code: `
            @group(0) @binding(0) var s: texture_storage_2d<${format}, read>;
            @compute @workgroup_size(1) fn cs() {
                _ = textureLoad(s, vec2u(0));
            }
          `
    }),
    true
  );
});

const kDepthTextureTypeToParams = {
  texture_depth_2d: 'vec2u(0), 0',
  texture_depth_2d_array: 'vec2u(0), 0, 0',
  texture_depth_multisampled_2d: 'vec2u(0), 0'
};
const kDepthTextureTypes = keysOf(kDepthTextureTypeToParams);

g.test('textureLoad_with_depth_textures').
desc(
  `Tests that you can not create a shader module that uses textureLoad with a depth texture in compat mode.`
).
params((u) => u.combine('type', kDepthTextureTypes)).
fn((t) => {
  const { type } = t.params;
  const params = kDepthTextureTypeToParams[type];

  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () =>
    t.device.createShaderModule({
      code: `
            @group(0) @binding(0) var t: ${type};
            @compute @workgroup_size(1) fn cs() {
                _ = textureLoad(t, ${params});
            }
          `
    }),
    true
  );
});