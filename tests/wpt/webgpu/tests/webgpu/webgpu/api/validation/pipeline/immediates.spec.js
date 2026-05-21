/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Pipeline creation validation tests for immediate data size mismatches.

Validates that creating a pipeline fails if the shader uses immediate data
larger than the immediateSize specified in the pipeline layout, or larger than
maxImmediateSize if layout is 'auto'.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { getGPU } from '../../../../common/util/navigator_gpu.js';
import { assert, range, supportsImmediateData } from '../../../../common/util/util.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';
import * as vtu from '../validation_test_utils.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

/**
 * Generate shader code for a given stage with the specified immediate data size.
 * If size is 0, the shader has no immediate data.
 */
function makeShaderCode(size, stage) {
  if (size === 0) {
    switch (stage) {
      case 'compute':
        return `@compute @workgroup_size(1) fn main_compute() {}`;
      case 'vertex':
        return `@vertex fn main_vertex() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }`;
      case 'fragment':
        return `@fragment fn main_fragment() -> @location(0) vec4<f32> { return vec4<f32>(0.0, 1.0, 0.0, 1.0); }`;
    }
  }
  const numFields = size / 4;
  const fields = range(numFields, (i) => `m${i}: u32`).join(', ');
  const structDecl = `struct Immediates { ${fields} }\nvar<immediate> data: Immediates;`;
  switch (stage) {
    case 'compute':
      return `${structDecl}\nfn use_data() { _ = data.m0; }\n@compute @workgroup_size(1) fn main_compute() { use_data(); }`;
    case 'vertex':
      return `${structDecl}\n@vertex fn main_vertex() -> @builtin(position) vec4<f32> { _ = data.m0; return vec4<f32>(0.0, 0.0, 0.0, 1.0); }`;
    case 'fragment':
      return `${structDecl}\n@fragment fn main_fragment() -> @location(0) vec4<f32> { _ = data.m0; return vec4<f32>(0.0, 1.0, 0.0, 1.0); }`;
  }
}

g.test('pipeline_creation_immediate_size_mismatch').
desc(
  `
    Validate that creating a compute or render pipeline fails if the shader uses
    immediate data larger than the immediateSize specified in the pipeline layout,
    or larger than maxImmediateSize if layout is 'auto'.
    Also validates that using less or equal size is allowed.

    For compute pipelines, stageASize is the compute stage size (stageBSize is unused).
    For render pipelines, stageASize is the vertex stage size and stageBSize is the
    fragment stage size.
    `
).
params((u) =>
u.
combine('pipelineType', ['compute', 'render']).
combine('isAsync', [true, false]).
beginSubcases().
expandWithParams(function* (p) {
  yield { stageASize: 16, stageBSize: 16, layoutSize: 16 }; // Equal
  yield { stageASize: 12, stageBSize: 12, layoutSize: 16 }; // Shader smaller
  yield { stageASize: 20, stageBSize: 20, layoutSize: 16 }; // Shader larger (small diff)
  yield { stageASize: 32, stageBSize: 32, layoutSize: 16 }; // Shader larger
  yield { stageASize: 'max', stageBSize: 0, layoutSize: 'auto' }; // StageA at limit
  yield { stageASize: 'exceedLimits', stageBSize: 0, layoutSize: 'auto' }; // StageA exceeds

  if (p.pipelineType === 'render') {
    yield { stageASize: 0, stageBSize: 'max', layoutSize: 'auto' }; // StageB at limit
    yield { stageASize: 'max', stageBSize: 'max', layoutSize: 'auto' }; // Both at limit
    yield { stageASize: 0, stageBSize: 'exceedLimits', layoutSize: 'auto' }; // StageB exceeds
  }
})
).
fn((t) => {
  t.skipIf(!supportsImmediateData(getGPU(t.rec)), 'Immediate data not supported');

  const { pipelineType, isAsync, stageASize, stageBSize, layoutSize } = t.params;

  const maxImmediateSize = t.device.limits.maxImmediateSize;
  assert(maxImmediateSize !== undefined);

  const resolveSize = (sizeDescriptor) => {
    if (typeof sizeDescriptor === 'number') return sizeDescriptor;
    if (sizeDescriptor === 'max') return maxImmediateSize;
    if (sizeDescriptor === 'exceedLimits') return maxImmediateSize + 4;
    return 0;
  };

  const resolvedStageASize = resolveSize(stageASize);
  const resolvedStageBSize = resolveSize(stageBSize);

  // Ensure the test's fixed sizes fit within the device limit.
  if (stageASize !== 'exceedLimits') {
    assert(
      resolvedStageASize <= maxImmediateSize,
      `stageASize (${resolvedStageASize}) must be <= maxImmediateSize (${maxImmediateSize})`
    );
  }
  if (stageBSize !== 'exceedLimits') {
    assert(
      resolvedStageBSize <= maxImmediateSize,
      `stageBSize (${resolvedStageBSize}) must be <= maxImmediateSize (${maxImmediateSize})`
    );
  }

  // Build pipeline layout.
  let layout;
  let validSize;

  if (layoutSize === 'auto') {
    layout = 'auto';
    validSize = maxImmediateSize;
  } else {
    layout = t.device.createPipelineLayout({
      bindGroupLayouts: [],
      immediateSize: layoutSize
    });
    validSize = layoutSize;
  }

  const stageAExceedsLimit = resolvedStageASize > validSize;
  const stageBExceedsLimit = resolvedStageBSize > validSize;
  const shouldError = stageAExceedsLimit || stageBExceedsLimit;

  if (pipelineType === 'compute') {
    const code = makeShaderCode(resolvedStageASize, 'compute');

    vtu.doCreateComputePipelineTest(t, isAsync, !shouldError, {
      layout,
      compute: { module: t.device.createShaderModule({ code }) }
    });
  } else {
    const vertexCode = makeShaderCode(resolvedStageASize, 'vertex');
    const fragmentCode = makeShaderCode(resolvedStageBSize, 'fragment');

    vtu.doCreateRenderPipelineTest(t, isAsync, !shouldError, {
      layout,
      vertex: { module: t.device.createShaderModule({ code: vertexCode }) },
      fragment: {
        module: t.device.createShaderModule({ code: fragmentCode }),
        targets: [{ format: 'rgba8unorm' }]
      }
    });
  }
});