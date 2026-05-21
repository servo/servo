/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, unreachable } from '../../common/util/util.js';import { getBlockInfoForTextureFormat,
isDepthOrStencilTextureFormat,
isDepthStencilTextureFormat,
isDepthTextureFormat,
isSintOrUintFormat,
isStencilTextureFormat } from
'../format_info.js';


import { getTextureCopyLayout } from './texture/layout.js';

import { reifyExtent3D, reifyOrigin3D } from './unions.js';

// Note: For values that are supposedly unused we use 0.123 as a sentinel for
// float formats and 123 for integer formats. For example, rendering to r8unorm
// returns (v, 9.123, 0.123, 0.123). Since only v should be used this shouldn't
// matter but just in case we set it to 123 so it's more likely to cause an
// issue if something is wrong.
const kLoadValueFromStorageInfo =





{
  r8snorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
    return vec4f(unpack4x8snorm(getSrc(byteOffset / 4))[byteOffset % 4], 0.123, 0.123, 0.123)
  `
  },
  r8unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
    return vec4f(unpack4x8unorm(getSrc(byteOffset / 4))[byteOffset % 4], 0.123, 0.123, 0.123)
  `
  },
  r8uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
    return vec4u(unpack4xU8(getSrc(byteOffset / 4))[byteOffset % 4], 123, 123, 123)
  `
  },
  r8sint: {
    storageType: 'u32',
    texelType: 'vec4i',
    unpackWGSL: `
    return vec4i(unpack4xI8(getSrc(byteOffset / 4))[byteOffset % 4], 123, 123, 123)
  `
  },
  rg8snorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
    let v = unpack4x8snorm(getSrc(byteOffset / 4));
    return vec4f(select(v.rg, v.ba, byteOffset % 4 >= 2), 0.123, 0.123)
  `
  },
  rg8unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
    let v = unpack4x8unorm(getSrc(byteOffset / 4));
    return vec4f(select(v.rg, v.ba, byteOffset % 4 >= 2), 0.123, 0.123)
  `
  },
  rg8uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
    let v = unpack4xU8(getSrc(byteOffset / 4));
    return vec4u(select(v.rg, v.ba, byteOffset % 4 >= 2), 123, 123)
  `
  },
  rg8sint: {
    storageType: 'u32',
    texelType: 'vec4i',
    unpackWGSL: `
    let v = unpack4xI8(getSrc(byteOffset / 4));
    return vec4i(select(v.rg, v.ba, byteOffset % 4 >= 2), 123, 123)
  `
  },
  rgba8snorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: 'return unpack4x8snorm(getSrc(byteOffset / 4))'
  },
  rgba8unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: 'return unpack4x8unorm(getSrc(byteOffset / 4))'
  },
  'rgba8unorm-srgb': {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
      let v = unpack4x8unorm(getSrc(byteOffset / 4));
      let srgb = select(
        v / 12.92,
        pow((v + 0.055) / 1.055, vec4f(2.4)),
        v >= vec4f(0.04045)
      );
      return vec4f(srgb.rgb, v.a);
    `
  },
  bgra8unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: 'return unpack4x8unorm(getSrc(byteOffset / 4)).bgra'
  },
  'bgra8unorm-srgb': {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
      let v = unpack4x8unorm(getSrc(byteOffset / 4));
      let srgb = select(
        v / 12.92,
        pow((v + 0.055) / 1.055, vec4f(2.4)),
        v >= vec4f(0.04045)
      );
      return vec4f(srgb.bgr, v.a);
    `
  },
  rgba8uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: 'return unpack4xU8(getSrc(byteOffset / 4))'
  },
  rgba8sint: {
    storageType: 'u32',
    texelType: 'vec4i',
    unpackWGSL: 'return unpack4xI8(getSrc(byteOffset / 4))'
  },
  rg11b10ufloat: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `return unpackRG11B10UFloat(getSrc(byteOffset / 4))`
  },
  r16float: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL:
    'return vec4f(unpack2x16float(getSrc(byteOffset / 4))[byteOffset % 4 / 2], 0.123, 0.123, 0.123)'
  },
  r16uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL:
    'return vec4u(extractBits(getSrc(byteOffset / 4), (byteOffset % 4 / 2 * 16), 16), 123, 123, 123)'
  },
  r16sint: {
    storageType: 'i32',
    texelType: 'vec4i',
    unpackWGSL:
    'return vec4i(extractBits(getSrc(byteOffset / 4), byteOffset % 4 / 2 * 16, 16), 123, 123, 123)'
  },
  rg16float: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: 'return vec4f(unpack2x16float(getSrc(byteOffset / 4)), 0.123, 0.123)'
  },
  rg16uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
      let v = getSrc(byteOffset / 4);
      return vec4u(v & 0xFFFF, v >> 16, 123, 123)
    `
  },
  rg16sint: {
    storageType: 'i32',
    texelType: 'vec4i',
    unpackWGSL: `
      let v = getSrc(byteOffset / 4);
      return vec4i(
        extractBits(v, 0, 16),
        extractBits(v, 16, 16),
        123, 123)
    `
  },
  rgba16float: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
      return vec4f(
        unpack2x16float(getSrc(byteOffset / 4)),
        unpack2x16float(getSrc(byteOffset / 4 + 1)))
    `
  },
  rgba16uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
      let v0 = getSrc(byteOffset / 4);
      let v1 = getSrc(byteOffset / 4 + 1);
      return vec4u(v0 & 0xFFFF, v0 >> 16, v1 & 0xFFFF, v1 >> 16)
    `
  },
  rgba16sint: {
    storageType: 'i32',
    texelType: 'vec4i',
    unpackWGSL: `
      let v0 = getSrc(byteOffset / 4);
      let v1 = getSrc(byteOffset / 4 + 1);
      return vec4i(
        extractBits(v0, 0, 16),
        extractBits(v0, 16, 16),
        extractBits(v1, 0, 16),
        extractBits(v1, 16, 16),
      )
    `
  },
  r16unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
    let raw = extractBits(getSrc(byteOffset / 4), (byteOffset % 4 / 2) * 16, 16);
    return vec4f(f32(raw) / 65535.0, 0.123, 0.123, 0.123);
  `
  },
  r16snorm: {
    storageType: 'i32',
    texelType: 'vec4f',
    unpackWGSL: `
    let raw = extractBits(getSrc(byteOffset / 4), (byteOffset % 4 / 2) * 16, 16);
    let signedVal = i32(raw);
    return vec4f(f32(signedVal) / 32767.0, 0.123, 0.123, 0.123);
  `
  },
  rg16unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
    let v = getSrc(byteOffset / 4);
    let r = extractBits(v, 0, 16);
    let g = extractBits(v, 16, 16);
    return vec4f(f32(r) / 65535.0, f32(g) / 65535.0, 0.123, 0.123);
  `
  },
  rg16snorm: {
    storageType: 'i32',
    texelType: 'vec4f',
    unpackWGSL: `
    let v = getSrc(byteOffset / 4);
    let r = i32(extractBits(v, 0, 16));
    let g = i32(extractBits(v, 16, 16));
    return vec4f(f32(r) / 32767.0, f32(g) / 32767.0, 0.123, 0.123);
  `
  },
  rgba16unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
    let v0 = getSrc(byteOffset / 4);
    let v1 = getSrc(byteOffset / 4 + 1);
    let r = extractBits(v0, 0, 16);
    let g = extractBits(v0, 16, 16);
    let b = extractBits(v1, 0, 16);
    let a = extractBits(v1, 16, 16);
    return vec4f(
      f32(r) / 65535.0,
      f32(g) / 65535.0,
      f32(b) / 65535.0,
      f32(a) / 65535.0
    );
  `
  },
  rgba16snorm: {
    storageType: 'i32',
    texelType: 'vec4f',
    unpackWGSL: `
    let v0 = getSrc(byteOffset / 4);
    let v1 = getSrc(byteOffset / 4 + 1);
    let r = i32(extractBits(v0, 0, 16));
    let g = i32(extractBits(v0, 16, 16));
    let b = i32(extractBits(v1, 0, 16));
    let a = i32(extractBits(v1, 16, 16));
    return vec4f(
      f32(r) / 32767.0,
      f32(g) / 32767.0,
      f32(b) / 32767.0,
      f32(a) / 32767.0
    );
  `
  },
  r32float: {
    storageType: 'f32',
    texelType: 'vec4f',
    unpackWGSL: 'return vec4f(getSrc(byteOffset / 4), 0.123, 0.123, 0.123)'
  },
  rgb10a2uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
      let v = getSrc(byteOffset / 4);
      return vec4u(
        extractBits(v, 0, 10),
        extractBits(v, 10, 10),
        extractBits(v, 20, 10),
        extractBits(v, 30, 2),
      )
    `
  },
  rgb10a2unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
      let v = getSrc(byteOffset / 4);
      return vec4f(
        f32(extractBits(v, 0, 10)) / f32(0x3FF),
        f32(extractBits(v, 10, 10)) / f32(0x3FF),
        f32(extractBits(v, 20, 10)) / f32(0x3FF),
        f32(extractBits(v, 30, 2)) / f32(0x3),
      )
    `
  },
  depth16unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
      let v = unpack2x16unorm(getSrc(byteOffset / 4))[byteOffset % 4 / 2];
      return vec4f(v, 0.123, 0.123, 0.123)
    `
  },
  depth32float: {
    storageType: 'f32',
    texelType: 'vec4f',
    unpackWGSL: `
      let v = getSrc(byteOffset / 4);
      return vec4f(v, 0.123, 0.123, 0.123)
    `
  },
  stencil8: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
      return vec4u(unpack4xU8(getSrc(byteOffset / 4))[byteOffset % 4], 123, 123, 123)
    `
  }
};

function getDepthStencilOptionsForFormat(
format,
aspect)
{
  return {
    useFragDepth:
    isDepthTextureFormat(format) && (!aspect || aspect === 'all' || aspect === 'depth-only'),
    discardWithStencil:
    isStencilTextureFormat(format) && (!aspect || aspect === 'all' || aspect === 'stencil-only')
  };
}

function getCopyBufferToTextureViaRenderCode(
srcFormat,
dstFormat,
dstAspect)
{
  const info = kLoadValueFromStorageInfo[srcFormat];
  assert(!!info);
  const { storageType, texelType, unpackWGSL } = info;
  const { useFragDepth, discardWithStencil } = getDepthStencilOptionsForFormat(
    dstFormat,
    dstAspect
  );
  assert(!useFragDepth || !discardWithStencil, 'can not do both aspects at once');

  const [depthDecl, depthCode] = useFragDepth ?
  ['@builtin(frag_depth) d: f32,', 'fs.d = fs.v[0];'] :
  ['', ''];

  const stencilCode = discardWithStencil ? 'if ((fs.v.r & vin.stencilMask) == 0) { discard; }' : '';

  const code = `
    struct Uniforms {
      numTexelRows: u32,
      bytesPerRow: u32,
      bytesPerSample: u32,
      sampleCount: u32,
      offset: u32,
    };

    struct VSOutput {
      @builtin(position) pos: vec4f,
      @location(0) @interpolate(flat, either) sampleIndex: u32,
      @location(1) @interpolate(flat, either) stencilMask: u32,
    };

    @vertex fn vs(@builtin(vertex_index) vNdx: u32, @builtin(instance_index) iNdx: u32) -> VSOutput {
      let points = array(
        vec2f(0, 0), vec2f(1, 0), vec2f(0, 1), vec2f(1, 1),
      );
      let sampleRow = vNdx / 4;
      let numSampleRows = f32(uni.numTexelRows * uni.sampleCount);
      let rowOffset = f32(sampleRow) / numSampleRows;
      let rowMult = 1.0 / numSampleRows;
      let p = (points[vNdx % 4] * vec2f(1, rowMult) + vec2f(0, rowOffset)) * 2.0 - 1.0;
      return VSOutput(
        vec4f(p, 0, 1),
        uni.sampleCount - sampleRow % uni.sampleCount - 1,
        1u << iNdx);
    }

    @group(0) @binding(0) var<uniform> uni: Uniforms;
    @group(0) @binding(1) var src: texture_2d<${storageType}>;

    // get a u32/i32/f32 from a r32uint/r32sint/r32float as though it was 1d array
    fn getSrc(offset: u32) -> ${storageType} {
      let width = textureDimensions(src, 0).x;
      let x = offset % width;
      let y = offset / width;
      return textureLoad(src, vec2u(x, y), 0).r;
    }

    const kFloat32FormatMantissaBits = 23;
    const kFloat32FormatBias = 127;
    fn floatBitsToNumber(
        rawBits: u32,
        bitOffset: u32,
        exponentBits: u32,
        mantissaBits: u32,
        bias: u32,
        signed: bool) -> f32 {
      let nonSignBits = exponentBits + mantissaBits;
      let allBits = nonSignBits + select(0u, 1u, signed);
      let allMask = (1u << allBits) - 1u;
      let bits = (rawBits >> bitOffset) & allMask;
      let nonSignBitsMask = (1u << nonSignBits) - 1u;
      let exponentAndMantissaBits = bits & nonSignBitsMask;
      let exponentMask = ((1u << exponentBits) - 1u) << mantissaBits;
      let infinityOrNaN = (bits & exponentMask) == exponentMask;
      if (infinityOrNaN) {
        let mantissaMask = (1u << mantissaBits) - 1;
        let signBit = 1u << nonSignBits;
        let isNegative = (bits & signBit) != 0;
        if ((bits & mantissaMask) != 0u) {
          return 0.0; // NaN (does not exist in WGSL)
        }
        if (isNegative) {
          return f32(-2e38); // NEGATIVE_INFINITY (does not exist in WGSL)
        } else {
          return f32(2e38); // POSITIVE_INFINITY (does not exist in WGSL)
        }
      }
      var f32BitsWithWrongBias =
        exponentAndMantissaBits << (kFloat32FormatMantissaBits - mantissaBits);
      // add in the sign
      f32BitsWithWrongBias |= (bits << (31u - nonSignBits)) & 0x80000000u;
      let numberWithWrongBias = bitcast<f32>(f32BitsWithWrongBias);
      return numberWithWrongBias * pow(2.0f, f32(kFloat32FormatBias - bias));
    }

    fn unpackRG11B10UFloat(v: u32) -> vec4f {
      return vec4f(
        floatBitsToNumber(v,  0, 5, 6, 15, false),
        floatBitsToNumber(v, 11, 5, 6, 15, false),
        floatBitsToNumber(v, 22, 5, 5, 15, false),
        1
      );
    }

    fn unpack(byteOffset: u32) -> ${texelType} {
      ${unpackWGSL};
    }

    struct FSOutput {
      @location(0) v: ${texelType},
      ${depthDecl}
    };

    @fragment fn fs(vin: VSOutput) -> FSOutput {
      let coord = vec2u(vin.pos.xy);
      let byteOffset =
        uni.offset +
        coord.y * uni.bytesPerRow +
        (coord.x * uni.sampleCount + vin.sampleIndex) * uni.bytesPerSample;
      var fs: FSOutput;
      fs.v = unpack(byteOffset);
      ${depthCode}
      ${stencilCode}
      return fs;
    }
    `;

  let dataFormat;
  switch (storageType) {
    case 'f32':
      dataFormat = 'r32float';
      break;
    case 'i32':
      dataFormat = 'r32sint';
      break;
    case 'u32':
      dataFormat = 'r32uint';
      break;
    default:
      unreachable();
  }
  return { code, dataFormat };
}

const s_copyBufferToTextureViaRenderPipelines = new WeakMap(


);

// This function emulates copyBufferToTexture by by rendering into the texture.
// This is for formats that can't be copied to directly. depth textures, stencil
// textures, multisampled textures.
//
// For source data it creates an r32uint/r32sint/r32float texture
// and copies the source buffer into it and then reads the texture
// as a 1d array. It does this because compat mode might not have
// storage buffers in fragment shaders.
function copyBufferToTextureViaRender(
t,
encoder,
source,
sourceFormat,
dest,
size)
{
  const { format: textureFormat, sampleCount } = dest.texture;
  const origin = reifyOrigin3D(dest.origin ?? [0]);
  const copySize = reifyExtent3D(size);
  const { useFragDepth, discardWithStencil } = getDepthStencilOptionsForFormat(
    dest.texture.format,
    dest.aspect
  );
  const resourcesToDestroy = [];

  const { device } = t;
  const numBlits = discardWithStencil ? 8 : 1;
  for (let blitCount = 0; blitCount < numBlits; ++blitCount) {
    const { code, dataFormat } = getCopyBufferToTextureViaRenderCode(
      sourceFormat,
      dest.texture.format,
      dest.aspect
    );
    const stencilWriteMask = 1 << blitCount;
    const id = JSON.stringify({
      textureFormat,
      sourceFormat,
      useFragDepth,
      stencilWriteMask,
      discardWithStencil,
      sampleCount,
      code
    });
    const pipelines =
    s_copyBufferToTextureViaRenderPipelines.get(device) ?? new Map();
    s_copyBufferToTextureViaRenderPipelines.set(device, pipelines);
    let pipeline = pipelines.get(id);
    if (!pipeline) {
      const module = device.createShaderModule({ code });
      pipeline = device.createRenderPipeline({
        label: `blitCopyFor-${textureFormat}`,
        layout: 'auto',
        vertex: { module },
        ...(discardWithStencil ?
        {
          fragment: {
            module,
            targets: []
          },
          depthStencil: {
            depthWriteEnabled: false,
            depthCompare: 'always',
            format: textureFormat,
            stencilWriteMask,
            stencilFront: {
              passOp: 'replace'
            }
          }
        } :
        useFragDepth ?
        {
          fragment: {
            module,
            targets: []
          },
          depthStencil: {
            depthWriteEnabled: true,
            depthCompare: 'always',
            format: textureFormat
          }
        } :
        {
          fragment: {
            module,
            targets: [{ format: textureFormat }]
          }
        }),
        primitive: {
          topology: 'triangle-strip'
        },
        ...(sampleCount > 1 && { multisample: { count: sampleCount } })
      });
      pipelines.set(id, pipeline);
    }

    const width = 1024;
    const bytesPerRow = width * 4;
    const fullRows = Math.floor(source.buffer.size / bytesPerRow);
    const rows = Math.ceil(source.buffer.size / bytesPerRow);
    const srcTexture = t.createTextureTracked({
      format: dataFormat,
      size: [width, rows],
      usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
    });
    resourcesToDestroy.push(srcTexture);

    if (fullRows > 0) {
      encoder.copyBufferToTexture({ buffer: source.buffer, bytesPerRow }, { texture: srcTexture }, [
      width,
      fullRows]
      );
    }
    if (rows > fullRows) {
      const totalPixels = source.buffer.size / 4;
      const pixelsCopied = fullRows * width;
      const pixelsInLastRow = totalPixels - pixelsCopied;
      encoder.copyBufferToTexture(
        {
          buffer: source.buffer,
          offset: pixelsCopied * 4,
          bytesPerRow
        },
        {
          texture: srcTexture,
          origin: [0, fullRows]
        },
        [pixelsInLastRow, 1]
      );
    }
    const baseMipLevel = dest.mipLevel;
    for (let l = 0; l < copySize.depthOrArrayLayers; ++l) {
      const baseArrayLayer = origin.z + l;
      const mipLevelCount = 1;
      const arrayLayerCount = 1;
      const pass = encoder.beginRenderPass(
        discardWithStencil ?
        {
          colorAttachments: [],
          depthStencilAttachment: {
            view: dest.texture.createView({
              baseMipLevel,
              baseArrayLayer,
              mipLevelCount,
              arrayLayerCount
            }),
            depthReadOnly: true,
            stencilClearValue: 0,
            stencilLoadOp: 'load',
            stencilStoreOp: 'store'
          }
        } :
        useFragDepth ?
        {
          colorAttachments: [],
          depthStencilAttachment: {
            view: dest.texture.createView({
              baseMipLevel,
              baseArrayLayer,
              mipLevelCount,
              arrayLayerCount
            }),
            depthClearValue: 0,
            depthLoadOp: 'clear',
            depthStoreOp: 'store',
            stencilReadOnly: true
          }
        } :
        {
          colorAttachments: [
          {
            view: dest.texture.createView({
              baseMipLevel,
              baseArrayLayer,
              mipLevelCount,
              arrayLayerCount
            }),
            loadOp: 'clear',
            storeOp: 'store'
          }]

        }
      );
      pass.setViewport(origin.x, origin.y, copySize.width, copySize.height, 0, 1);
      pass.setPipeline(pipeline);

      const info = getBlockInfoForTextureFormat(sourceFormat);
      const offset =
      (source.offset ?? 0) + (source.bytesPerRow ?? 0) * (source.rowsPerImage ?? 0) * l;
      const uniforms = new Uint32Array([
      copySize.height, //  numTexelRows: u32,
      source.bytesPerRow, //  bytesPerRow: u32,
      info.bytesPerBlock, //  bytesPerSample: u32,
      dest.texture.sampleCount, //  sampleCount: u32,
      offset //  offset: u32,
      ]);

      const uniformBuffer = t.makeBufferWithContents(
        uniforms,
        GPUBufferUsage.COPY_DST | GPUBufferUsage.UNIFORM
      );
      resourcesToDestroy.push(uniformBuffer);
      const bindGroup = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
        { binding: 0, resource: { buffer: uniformBuffer } },
        { binding: 1, resource: srcTexture.createView() }]

      });

      pass.setBindGroup(0, bindGroup);
      pass.setStencilReference(0xff);
      pass.draw(4 * copySize.height * dest.texture.sampleCount, 1, 0, blitCount);
      pass.end();
    }
  }

  return resourcesToDestroy;
}

/**
 * Creates a mipmapped texture where each mipmap level's (`i`) content is
 * from `texelViews[i]`.
 */
export function createTextureFromTexelViews(
t,
texelViews,
desc)
{
  // All texel views must be the same format for mipmaps.
  assert(texelViews.length > 0 && texelViews.every((e) => e.format === texelViews[0].format));
  const viewsFormat = texelViews[0].format;
  const textureFormat = desc.format ?? viewsFormat;

  // Create the texture and then initialize each mipmap level separately.
  const texture = t.createTextureTracked({
    ...desc,
    format: textureFormat,
    usage: desc.usage | GPUTextureUsage.COPY_DST,
    mipLevelCount: texelViews.length
  });
  // Note: At the time of this writing there is no such thing as a depth-stencil TexelView
  // so we couldn't have passed in data for "all" aspects. This seems like a code smell issue
  // but it's a big change to fix.
  const aspect = isDepthStencilTextureFormat(textureFormat) ?
  isSintOrUintFormat(viewsFormat) ?
  'stencil-only' :
  'depth-only' :
  'all';
  copyTexelViewsToTexture(t, texture, aspect, texelViews);
  return texture;
}

export function copyTexelViewsToTexture(
t,
texture,
aspect,
texelViews)
{
  const viewsFormat = texelViews[0].format;
  const isTextureFormatDifferentThanTexelViewFormat = texture.format !== viewsFormat;
  const { width, height, depthOrArrayLayers } = texture;

  // Copy the texel view into each mip level layer.
  const commandEncoder = t.device.createCommandEncoder({ label: 'copyTexelViewToTexture' });
  const resourcesToDestroy = [];
  for (let mipLevel = 0; mipLevel < texelViews.length; mipLevel++) {
    const {
      bytesPerRow,
      rowsPerImage,
      mipSize: [mipWidth, mipHeight, mipDepthOrArray]
    } = getTextureCopyLayout(
      viewsFormat,
      texture.dimension ?? '2d',
      [width, height, depthOrArrayLayers],
      {
        mipLevel
      }
    );

    // Create a staging buffer to upload the texture mip level contents.
    const stagingBuffer = t.createBufferTracked({
      mappedAtCreation: true,
      size: bytesPerRow * mipHeight * mipDepthOrArray,
      usage: GPUBufferUsage.COPY_SRC
    });
    resourcesToDestroy.push(stagingBuffer);

    // Write the texels into the staging buffer.
    texelViews[mipLevel].writeTextureData(new Uint8Array(stagingBuffer.getMappedRange()), {
      bytesPerRow,
      rowsPerImage: mipHeight,
      subrectOrigin: [0, 0, 0],
      subrectSize: [mipWidth, mipHeight, mipDepthOrArray],
      sampleCount: texture.sampleCount
    });
    stagingBuffer.unmap();

    const copyB2TOk =
    !isTextureFormatDifferentThanTexelViewFormat &&
    texture.sampleCount === 1 &&
    !isDepthOrStencilTextureFormat(texture.format);

    if (!copyB2TOk) {
      resourcesToDestroy.push(
        ...copyBufferToTextureViaRender(
          t,
          commandEncoder,
          { buffer: stagingBuffer, bytesPerRow, rowsPerImage },
          viewsFormat,
          { texture, mipLevel, aspect },
          [mipWidth, mipHeight, mipDepthOrArray]
        )
      );
    } else {
      // Copy from the staging buffer into the texture.
      commandEncoder.copyBufferToTexture(
        { buffer: stagingBuffer, bytesPerRow, rowsPerImage },
        { texture, mipLevel, aspect: aspect ?? 'all' },
        [mipWidth, mipHeight, mipDepthOrArray]
      );
    }
  }
  t.device.queue.submit([commandEncoder.finish()]);

  // Cleanup temp buffers and textures.
  resourcesToDestroy.forEach((value) => value.destroy());
}