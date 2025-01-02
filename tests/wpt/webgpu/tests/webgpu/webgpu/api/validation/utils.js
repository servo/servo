/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/







/**
 * Returns an array of possible resources
 */
function generateResources() {
  const resources = [
  // Buffers
  {
    buffer: { type: 'uniform' },
    code: `var<uniform> res : array<vec4u, 16>`,
    staticUse: `res[0]`
  },
  {
    buffer: { type: 'storage' },
    code: `var<storage, read_write> res : array<vec4u>`,
    staticUse: `res[0]`
  },
  {
    buffer: { type: 'read-only-storage' },
    code: `var<storage> res : array<vec4u>`,
    staticUse: `res[0]`
  },

  // Samplers
  {
    sampler: { type: 'filtering' },
    code: `var res : sampler`
  },
  {
    sampler: { type: 'non-filtering' },
    code: `var res : sampler`
  },
  {
    sampler: { type: 'comparison' },
    code: `var res : sampler_comparison`
  },
  // Multisampled textures
  {
    texture: { sampleType: 'depth', viewDimension: '2d', multisampled: true },
    code: `var res : texture_depth_multisampled_2d`
  },
  {
    texture: { sampleType: 'unfilterable-float', viewDimension: '2d', multisampled: true },
    code: `var res : texture_multisampled_2d<f32>`
  },
  {
    texture: { sampleType: 'sint', viewDimension: '2d', multisampled: true },
    code: `var res : texture_multisampled_2d<i32>`
  },
  {
    texture: { sampleType: 'uint', viewDimension: '2d', multisampled: true },
    code: `var res : texture_multisampled_2d<u32>`
  }];


  // Sampled textures
  const sampleDims = [
  '1d',
  '2d',
  '2d-array',
  '3d',
  'cube',
  'cube-array'];

  const sampleTypes = ['float', 'unfilterable-float', 'sint', 'uint'];
  const sampleWGSL = ['f32', 'f32', 'i32', 'u32'];
  for (const dim of sampleDims) {
    let i = 0;
    for (const type of sampleTypes) {
      resources.push({
        texture: { sampleType: type, viewDimension: dim, multisampled: false },
        code: `var res : texture_${dim.replace('-', '_')}<${sampleWGSL[i++]}>`
      });
    }
  }

  // Depth textures
  const depthDims = ['2d', '2d-array', 'cube', 'cube-array'];
  for (const dim of depthDims) {
    resources.push({
      texture: { sampleType: 'depth', viewDimension: dim, multisampled: false },
      code: `var res : texture_depth_${dim.replace('-', '_')}`
    });
  }

  // Storage textures
  // Only cover r32uint, r32sint, and r32float here for ease of testing.
  const storageDims = ['1d', '2d', '2d-array', '3d'];
  const formats = ['r32float', 'r32sint', 'r32uint'];
  const accesses = ['write-only', 'read-only', 'read-write'];
  for (const dim of storageDims) {
    for (const format of formats) {
      for (const access of accesses) {
        resources.push({
          storageTexture: { access, format, viewDimension: dim },
          code: `var res : texture_storage_${dim.replace('-', '_')}<${format},${access.
          replace('-only', '').
          replace('-', '_')}>`
        });
      }
    }
  }

  return resources;
}

/**
 * Returns a string suitable as a Record key.
 */
function resourceKey(res) {
  if (res.buffer) {
    return `${res.buffer.type}_buffer`;
  }
  if (res.sampler) {
    return `${res.sampler.type}_sampler`;
  }
  if (res.texture) {
    return `texture_${res.texture.sampleType}_${res.texture.viewDimension}_${res.texture.multisampled}`;
  }
  if (res.storageTexture) {
    return `storage_texture_${res.storageTexture.viewDimension}_${res.storageTexture.format}_${res.storageTexture.access}`;
  }
  if (res.externalTexture) {
    return `external_texture`;
  }
  return ``;
}

/**
 * Resource array converted to a Record for nicer test parameterization names.
 */
export const kAPIResources = Object.fromEntries(
  generateResources().map((x) => [resourceKey(x), x])
);

/**
 * Generates a shader of the specified stage using the specified resource at binding (0,0).
 */
export function getWGSLShaderForResource(stage, resource) {
  let code = `@group(0) @binding(0) ${resource.code};\n`;

  code += `@${stage}`;
  if (stage === 'compute') {
    code += `@workgroup_size(1)`;
  }

  let retTy = '';
  let retVal = '';
  if (stage === 'vertex') {
    retTy = ' -> @builtin(position) vec4f';
    retVal = 'return vec4f();';
  } else if (stage === 'fragment') {
    retTy = ' -> @location(0) vec4f';
    retVal = 'return vec4f();';
  }
  code += `
fn main() ${retTy} {
  _ = ${resource.staticUse ?? 'res'};
  ${retVal}
}
`;

  return code;
}

/**
 * Generates a bind group layout for for the given resource at binding 0.
 */
export function getAPIBindGroupLayoutForResource(
device,
stage,
resource)
{
  const entry = {
    binding: 0,
    visibility: stage
  };
  if (resource.buffer) {
    entry.buffer = resource.buffer;
  }
  if (resource.sampler) {
    entry.sampler = resource.sampler;
  }
  if (resource.texture) {
    entry.texture = resource.texture;
  }
  if (resource.storageTexture) {
    entry.storageTexture = resource.storageTexture;
  }
  if (resource.externalTexture) {
    entry.externalTexture = resource.externalTexture;
  }

  const entries = [entry];
  return device.createBindGroupLayout({ entries });
}

/**
 * Returns true if the sample types are compatible.
 */
function doSampleTypesMatch(api, wgsl) {
  if (api === 'float' || api === 'unfilterable-float') {
    return wgsl === 'float' || wgsl === 'unfilterable-float';
  }
  return api === wgsl;
}

/**
 * Returns true if the access modes are compatible.
 */
function doAccessesMatch(api, wgsl) {
  if (api === 'read-write') {
    return wgsl === 'read-write' || wgsl === 'write-only';
  }
  return api === wgsl;
}

/**
 * Returns true if the resources are compatible.
 */
export function doResourcesMatch(api, wgsl) {
  if (api.buffer) {
    if (!wgsl.buffer) {
      return false;
    }
    return api.buffer.type === wgsl.buffer.type;
  }
  if (api.sampler) {
    if (!wgsl.sampler) {
      return false;
    }
    return (
      api.sampler.type === wgsl.sampler.type ||
      api.sampler.type !== 'comparison' && wgsl.sampler.type !== 'comparison');

  }
  if (api.texture) {
    if (!wgsl.texture) {
      return false;
    }
    const aType = api.texture.sampleType;
    const wType = wgsl.texture.sampleType;
    return (
      doSampleTypesMatch(aType, wType) &&
      api.texture.viewDimension === wgsl.texture.viewDimension &&
      api.texture.multisampled === wgsl.texture.multisampled);

  }
  if (api.storageTexture) {
    if (!wgsl.storageTexture) {
      return false;
    }
    const aAccess = api.storageTexture.access;
    const wAccess = wgsl.storageTexture.access;
    return (
      doAccessesMatch(aAccess, wAccess) &&
      api.storageTexture.format === wgsl.storageTexture.format &&
      api.storageTexture.viewDimension === wgsl.storageTexture.viewDimension);

  }
  if (api.externalTexture) {
    return wgsl.externalTexture !== undefined;
  }

  return false;
}