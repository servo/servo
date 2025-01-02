/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for resource compatibilty between pipeline layout and shader modules
  `;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import {
  kAPIResources,
  getWGSLShaderForResource,
  getAPIBindGroupLayoutForResource,
  doResourcesMatch } from
'../utils.js';

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
    'Storage buffers and textures cannot be used in vertex shaders'
  );
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

  t.doCreateRenderPipelineTest(
    t.params.isAsync,
    doResourcesMatch(apiResource, wgslResource),
    descriptor
  );
});