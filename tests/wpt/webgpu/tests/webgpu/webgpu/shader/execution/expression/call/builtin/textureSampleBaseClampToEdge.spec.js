/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for textureSampleBaseClampToEdge
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';



import {
  checkCallResults,
  createTextureWithRandomDataAndGetTexels,
  createVideoFrameWithRandomDataAndGetTexels,
  doTextureCalls,
  generateTextureBuiltinInputs2D,
  kSamplePointMethods,
  kShortAddressModes,
  kShortAddressModeToAddressMode,
  kShortShaderStages,


  WGSLTextureSampleTest } from
'./texture_utils.js';

export const g = makeTestGroup(WGSLTextureSampleTest);

async function createTextureAndDataForTest(
t,
descriptor,
isExternal)




{
  if (isExternal) {
    const { texels, videoFrame } = createVideoFrameWithRandomDataAndGetTexels(descriptor.size);
    const texture = t.device.importExternalTexture({ source: videoFrame });
    return { texels, texture, videoFrame };
  } else {
    return await createTextureWithRandomDataAndGetTexels(t, descriptor);
  }
}

g.test('2d_coords').
specURL('https://www.w3.org/TR/WGSL/#texturesamplebaseclamptoedge').
desc(
  `
fn textureSampleBaseClampToEdge(t: texture_2d<f32>, s: sampler, coords: vec2<f32>) -> vec4<f32>
fn textureSampleBaseClampToEdge(t: texture_external, s: sampler, coords: vec2<f32>) -> vec4<f32>


Parameters:
 * t  The texture to sample.
 * s  The sampler type.
 * coords The texture coordinates used for sampling.
`
).
params((u) =>
u.
combine('stage', kShortShaderStages).
combine('textureType', ['texture_2d<f32>', 'texture_external']).
combine('filt', ['nearest', 'linear']).
combine('modeU', kShortAddressModes).
combine('modeV', kShortAddressModes).
beginSubcases().
combine('samplePoints', kSamplePointMethods)
).
beforeAllSubcases((t) =>
t.skipIf(
  t.params.textureType === 'texture_external' && typeof VideoFrame === 'undefined',
  'VideoFrames are not supported'
)
).
fn(async (t) => {
  const { textureType, stage, samplePoints, modeU, modeV, filt: minFilter } = t.params;

  const descriptor = {
    format: 'rgba8unorm',
    size: [8, 8],
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    mipLevelCount: 3
  };

  const isExternal = textureType === 'texture_external';
  const { texture, texels, videoFrame } = await createTextureAndDataForTest(
    t,
    descriptor,
    isExternal
  );
  try {
    const sampler = {
      addressModeU: kShortAddressModeToAddressMode[modeU],
      addressModeV: kShortAddressModeToAddressMode[modeV],
      minFilter,
      magFilter: minFilter,
      mipmapFilter: minFilter
    };

    const calls = generateTextureBuiltinInputs2D(50, {
      method: samplePoints,
      sampler,
      descriptor,
      hashInputs: [samplePoints, modeU, modeV, minFilter]
    }).map(({ coords }) => {
      return {
        builtin: 'textureSampleBaseClampToEdge',
        coordType: 'f',
        coords
      };
    });
    const viewDescriptor = {};
    const results = await doTextureCalls(
      t,
      texture,
      viewDescriptor,
      textureType,
      sampler,
      calls,
      stage
    );
    const res = await checkCallResults(
      t,
      { texels, descriptor, viewDescriptor },
      textureType,
      sampler,
      calls,
      results,
      stage
    );
    t.expectOK(res);
  } finally {
    videoFrame?.close();
  }
});