/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that non-filterable textures used with filtering samplers generate a validation error.
`;import { AllFeaturesMaxLimitsGPUTest } from '../.././gpu_test.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { keysOf } from '../../../common/util/data_tables.js';

import * as vtu from './validation_test_utils.js';

const kNonFilterableCaseInfo = {
  sint: { type: 'i32', component: '0,' },
  uint: { type: 'u32', component: '0,' },
  float: { type: 'f32', component: '0,' }, // no error for f32
  'unfilterable-float': { type: 'f32', component: '0,' }, // no error for f32
  depth: { type: 'depth', component: '' }
};
const kNonFilterableCases = keysOf(kNonFilterableCaseInfo);

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('non_filterable_texture_with_filtering_sampler').
desc(
  'test that createXXXPipeline generates a validation error if a depth/u32/i32 texture binding is used with a filtering sampler binding'
).
params((u) =>
u.
combine('pipeline', ['compute', 'render']).
combine('async', [true, false]).
combine('sampleType', kNonFilterableCases).
combine('viewDimension', ['2d', '2d-array', 'cube', 'cube-array']).
combine('sameGroup', [true, false])
).
fn((t) => {
  const { device } = t;
  const { pipeline, async, sampleType, viewDimension, sameGroup } = t.params;
  const { type, component } = kNonFilterableCaseInfo[sampleType];
  t.skipIfTextureViewDimensionNotSupported(viewDimension);

  const coord = viewDimension.startsWith('2d') ? 'vec2f(0)' : 'vec3f(0)';
  const dimensionSuffix = viewDimension.replace('-', '_');
  const textureType =
  type === 'depth' ? `texture_depth_${dimensionSuffix}` : `texture_${dimensionSuffix}<${type}>`;
  const layer = viewDimension.endsWith('-array') ? ', 0' : '';

  const groupNdx = sameGroup ? 0 : 1;

  const module = device.createShaderModule({
    code: `
      @group(0) @binding(0) var t: ${textureType};
      @group(${groupNdx}) @binding(1) var s: sampler;

      fn test() {
        _ = textureGather(${component} t, s, ${coord}${layer});
      }

      @compute @workgroup_size(1) fn cs() {
        test();
      }

      @vertex fn vs() -> @builtin(position) vec4f {
        return vec4f(0);
      }

      @fragment fn fs() -> @location(0) vec4f {
        test();
        return vec4f(0);
      }
      `
  });

  const bindGroup0LayoutEntries = [
  {
    binding: 0,
    visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT,
    texture: {
      sampleType,
      viewDimension,
      multisampled: false
    }
  }];


  const samplerBGLEntry = {
    binding: 1,
    visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT,
    sampler: {
      type: 'filtering'
    }
  };

  if (sameGroup) {
    bindGroup0LayoutEntries.push(samplerBGLEntry);
  }

  const bindGroupLayout0 = device.createBindGroupLayout({
    entries: bindGroup0LayoutEntries
  });

  const pipelineLayoutDesc = {
    bindGroupLayouts: [bindGroupLayout0]
  };

  if (!sameGroup) {
    const bindGroupLayout1 = device.createBindGroupLayout({
      entries: [samplerBGLEntry]
    });
    pipelineLayoutDesc.bindGroupLayouts.push(bindGroupLayout1);
  }

  const layout = device.createPipelineLayout(pipelineLayoutDesc);

  const success = sampleType === 'float';

  if (pipeline === 'compute') {
    vtu.doCreateComputePipelineTest(t, async, success, {
      layout,
      compute: { module }
    });
  } else {
    vtu.doCreateRenderPipelineTest(t, async, success, {
      layout,
      vertex: { module },
      fragment: { module, targets: [{ format: 'rgba8unorm' }] }
    });
  }
});