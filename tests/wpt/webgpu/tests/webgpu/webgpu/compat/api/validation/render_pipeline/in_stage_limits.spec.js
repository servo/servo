/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that, in compat mode, you can not create a pipeline layout with with
more than the max in stage limit even if the per stage limit is higher.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { range } from '../../../../../common/util/util.js';
import * as vtu from '../../../../api/validation/validation_test_utils.js';
import { RequiredLimitsTestMixin } from '../../../../gpu_test.js';
import { CompatibilityTest } from '../../../compatibility_test.js';

export const g = makeTestGroup(
  RequiredLimitsTestMixin(CompatibilityTest, {
    getRequiredLimits(adapter) {
      return {
        maxStorageBuffersInFragmentStage: adapter.limits.maxStorageBuffersInFragmentStage / 2,
        maxStorageBuffersInVertexStage: adapter.limits.maxStorageBuffersInVertexStage / 2,
        maxStorageBuffersPerShaderStage: adapter.limits.maxStorageBuffersPerShaderStage,
        maxStorageTexturesInFragmentStage: adapter.limits.maxStorageTexturesInFragmentStage / 2,
        maxStorageTexturesInVertexStage: adapter.limits.maxStorageTexturesInVertexStage / 2,
        maxStorageTexturesPerShaderStage: adapter.limits.maxStorageTexturesPerShaderStage
      };
    },
    key() {
      return `
      maxStorageBuffersInFragmentStage/2,
      maxStorageBuffersInVertexStage/2,
      maxStorageTexturesInFragmentStage/2,
      maxStorageTexturesInVertexStage/2,
      maxStorageBuffersPerShaderStage
      maxStorageTexturesPerShaderStage
    `;
    }
  })
);

g.test('maxStorageBuffersTexturesInVertexFragmentStage').
desc(
  `
      Tests that you can't use more than maxStorage(Buffers/Textures)In(Fragment/Vertex)Stage when
      the limit is less than maxStorage(Buffers/Textures)PerShaderStage
    `
).
params((u) =>
u.
combine('limit', [
'maxStorageBuffersInFragmentStage',
'maxStorageBuffersInVertexStage',
'maxStorageTexturesInFragmentStage',
'maxStorageTexturesInVertexStage']
).
beginSubcases().
combine('async', [false, true]).
combine('extra', [0, 1])
).
fn((t) => {
  const { limit, extra, async } = t.params;
  const { device } = t;

  const isBuffer = limit.includes('Buffers');
  const inStageLimit = device.limits[limit];
  const perStageLimitName = isBuffer ?
  'maxStorageBuffersPerShaderStage' :
  'maxStorageTexturesPerShaderStage';
  const perStageLimit = device.limits[perStageLimitName];

  t.debug(`${limit}(${inStageLimit}), ${perStageLimitName}(${perStageLimit})`);

  t.skipIf(inStageLimit === 0, `${limit} is 0`);
  t.skipIf(
    !(inStageLimit < perStageLimit),
    `${limit}(${inStageLimit}) is not less than ${perStageLimitName}(${perStageLimit})`
  );

  const typeWGSLFn = isBuffer ?
  (i) => `var<storage, read> v${i}: f32;` :
  (i) => `var v${i}: texture_storage_2d<r32float, read>;`;

  const count = inStageLimit + extra;
  const code = `
    ${range(count, (i) => `@group(0) @binding(${i}) ${typeWGSLFn(i)}`).join('\n')}

    fn useResources() {
      ${range(count, (i) => `_ = v${i};`).join('\n')}
    }

    @vertex fn vsNoUse() -> @builtin(position) vec4f {
      return vec4f(0);
    }

    @vertex fn vsUse() -> @builtin(position) vec4f {
      useResources();
      return vec4f(0);
    }

    @fragment fn fsNoUse() -> @location(0) vec4f {
      return vec4f(0);
    }

    @fragment fn fsUse() -> @location(0) vec4f {
      useResources();
      return vec4f(0);
    }
    `;

  const module = device.createShaderModule({ code });

  const isFragment = limit.includes('Fragment');
  const pipelineDescriptor = {
    layout: 'auto',
    vertex: {
      module,
      entryPoint: isFragment ? 'vsNoUse' : 'vsUse'
    },
    fragment: {
      module,
      entryPoint: isFragment ? 'fsUse' : 'fsNoUse',
      targets: [{ format: 'rgba8unorm' }]
    }
  };

  const success = extra === 0;
  vtu.doCreateRenderPipelineTest(t, async, success, pipelineDescriptor);
});