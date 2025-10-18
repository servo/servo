/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for resource compatibility between pipeline layout and shader modules
  `;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import {
  kAPIResources,
  getWGSLShaderForResource,
  getAPIBindGroupLayoutForResource,
  doResourcesMatch } from
'../utils.js';
import * as vtu from '../validation_test_utils.js';

import { CreateRenderPipelineValidationTest } from './common.js';

export const g = makeTestGroup(CreateRenderPipelineValidationTest);

g.test('resource_compatibility').
desc(
  'Tests validation of resource (bind group) compatibility between pipeline layout and WGSL shader'
).
params((u) =>
u //
.combine('stage', ['vertex', 'fragment']).
combine('apiResource', keysOf(kAPIResources)).
filter((t) => {
  const res = kAPIResources[t.apiResource];
  if (t.stage === 'vertex') {
    if (res.buffer && res.buffer.type === 'storage') {
      return false;
    }
    if (res.storageTexture && res.storageTexture.access !== 'read-only') {
      return false;
    }
  }
  return true;
}).
beginSubcases().
combine('isAsync', [true, false]).
combine('wgslResource', keysOf(kAPIResources))
).
fn((t) => {
  const apiResource = kAPIResources[t.params.apiResource];
  const wgslResource = kAPIResources[t.params.wgslResource];
  t.skipIf(
    wgslResource.storageTexture !== undefined &&
    wgslResource.storageTexture.access !== 'write-only' &&
    !t.hasLanguageFeature('readonly_and_readwrite_storage_textures'),
    'Storage textures require language feature'
  );
  t.skipIf(
    t.params.stage === 'vertex' && (
    wgslResource.buffer !== undefined && wgslResource.buffer.type === 'storage' ||
    wgslResource.storageTexture !== undefined &&
    wgslResource.storageTexture.access !== 'read-only'),
    'Read-Write Storage buffers and textures cannot be used in vertex shaders'
  );
  if (t.isCompatibility) {
    t.skipIf(
      t.params.stage === 'vertex' && (
      apiResource.buffer?.type === 'storage' ||
      apiResource.buffer?.type === 'read-only-storage') &&
      t.device.limits.maxStorageBuffersInVertexStage === 0,
      'Storage buffers can not be used in vertex shaders because maxStorageBuffersInVertexStage === 0'
    );
    t.skipIf(
      t.params.stage === 'vertex' &&
      apiResource.storageTexture !== undefined &&
      t.device.limits.maxStorageTexturesInVertexStage === 0,
      'Storage textures can not be used in vertex shaders because maxStorageTexturesInVertexStage === 0'
    );
    t.skipIf(
      t.params.stage === 'fragment' && (
      apiResource.buffer?.type === 'storage' ||
      apiResource.buffer?.type === 'read-only-storage') &&
      t.device.limits.maxStorageBuffersInFragmentStage === 0,
      'Storage buffers can not be used in fragment shaders because maxStorageBuffersInFragmentStage === 0'
    );
    t.skipIf(
      t.params.stage === 'fragment' &&
      apiResource.storageTexture !== undefined &&
      t.device.limits.maxStorageTexturesInFragmentStage === 0,
      'Storage textures can not be used in fragment shaders because maxStorageTexturesInFragmentStage === 0'
    );
  }
  t.skipIfTextureViewDimensionNotSupported(wgslResource.texture?.viewDimension);
  const emptyVS = `
@vertex
fn main() -> @builtin(position) vec4f {
  return vec4f();
}
`;
  const emptyFS = `
@fragment
fn main() -> @location(0) vec4f {
  return vec4f();
}
`;

  const code = getWGSLShaderForResource(t.params.stage, wgslResource);
  const vsCode = t.params.stage === 'vertex' ? code : emptyVS;
  const fsCode = t.params.stage === 'fragment' ? code : emptyFS;
  const gpuStage =
  t.params.stage === 'vertex' ? GPUShaderStage.VERTEX : GPUShaderStage.FRAGMENT;
  const layout = t.device.createPipelineLayout({
    bindGroupLayouts: [getAPIBindGroupLayoutForResource(t.device, gpuStage, apiResource)]
  });

  const descriptor = {
    layout,
    vertex: {
      module: t.device.createShaderModule({
        code: vsCode
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: fsCode
      }),
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm' }]
    }
  };

  vtu.doCreateRenderPipelineTest(
    t,
    t.params.isAsync,
    doResourcesMatch(apiResource, wgslResource),
    descriptor
  );
});