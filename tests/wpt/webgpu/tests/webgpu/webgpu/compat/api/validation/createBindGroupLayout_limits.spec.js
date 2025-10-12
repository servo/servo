/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that, in compat mode, you can not create a bind group layout with with
more than the max in stage limit even if the per stage limit is higher.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { range } from '../../../../common/util/util.js';
import { RequiredLimitsTestMixin } from '../../../gpu_test.js';
import { CompatibilityTest } from '../../compatibility_test.js';

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
combine('extra', [0, 1])
).
fn((t) => {
  const { limit, extra } = t.params;
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

  const visibility = limit.includes('Fragment') ? GPUShaderStage.FRAGMENT : GPUShaderStage.VERTEX;

  const expectFailure = extra > 0;
  t.expectValidationError(() => {
    device.createBindGroupLayout({
      entries: range(inStageLimit + extra, (i) => ({
        binding: i,
        visibility,
        ...(isBuffer ?
        { buffer: { type: 'read-only-storage' } } :
        { storageTexture: { format: 'r32float', access: 'read-only' } })
      }))
    });
  }, expectFailure);
});