/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../common/util/util.js';import { isDepthOrStencilTextureFormat, kTextureFormatInfo } from '../format_info.js';

import { getTextureCopyLayout } from './texture/layout.js';

import { reifyExtent3D, reifyOrigin3D } from './unions.js';

// Note: For values that are supposedly unused we use 0.123 as a sentinel for
// float formats and 123 for integer formats. For example, rendering to r8unorm
// returns (v, 9.123, 0.123, 0.123). Since only v should be used this shouldn't
// matter but just in case we set it to 123 so it's more likely to cause an
// issue if something is wrong.
const kLoadValueFromStorageInfo =







{
  r8unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
    return vec4f(unpack4x8unorm(src[byteOffset / 4])[byteOffset % 4], 0.123, 0.123, 0.123)
  `
  },
  r8uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
    return vec4u(unpack4xU8(src[byteOffset / 4])[byteOffset % 4], 123, 123, 123)
  `
  },
  r8sint: {
    storageType: 'u32',
    texelType: 'vec4i',
    unpackWGSL: `
    return vec4i(unpack4xI8(src[byteOffset / 4])[byteOffset % 4], 123, 123, 123)
  `
  },
  rg8unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
    let v = unpack4x8unorm(src[byteOffset / 4]);
    return vec4f(select(v.rg, v.ba, byteOffset % 4 >= 2), 0.123, 0.123)
  `
  },
  rg8uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
    let v = unpack4xU8(src[byteOffset / 4]);
    return vec4u(select(v.rg, v.ba, byteOffset % 4 >= 2), 123, 123)
  `
  },
  rg8sint: {
    storageType: 'u32',
    texelType: 'vec4i',
    unpackWGSL: `
    let v = unpack4xI8(src[byteOffset / 4]);
    return vec4i(select(v.rg, v.ba, byteOffset % 4 >= 2), 123, 123)
  `
  },
  rgba8unorm: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: 'return unpack4x8unorm(src[byteOffset / 4])'
  },
  'rgba8unorm-srgb': {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
      let v = unpack4x8unorm(src[byteOffset / 4]);
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
    unpackWGSL: 'return unpack4x8unorm(src[byteOffset / 4]).bgra'
  },
  'bgra8unorm-srgb': {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: `
      let v = unpack4x8unorm(src[byteOffset / 4]);
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
    unpackWGSL: 'return unpack4xU8(src[byteOffset / 4])'
  },
  rgba8sint: {
    storageType: 'u32',
    texelType: 'vec4i',
    unpackWGSL: 'return unpack4xI8(src[byteOffset / 4])'
  },
  r16float: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL:
    'return vec4f(unpack2x16float(src[byteOffset / 4])[byteOffset % 4 / 2], 0.123, 0.123, 0.123)'
  },
  r16uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL:
    'return vec4u(extractBits(src[byteOffset / 4], (byteOffset % 4 / 2 * 16), 16), 123, 123, 123)'
  },
  r16sint: {
    storageType: 'i32',
    texelType: 'vec4i',
    unpackWGSL:
    'return vec4i(extractBits(src[byteOffset / 4], byteOffset % 4 / 2 * 16, 16), 123, 123, 123)'
  },
  rg16float: {
    storageType: 'u32',
    texelType: 'vec4f',
    unpackWGSL: 'return vec4f(unpack2x16float(src[byteOffset / 4]), 0.123, 0.123)'
  },
  rg16uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
      let v = src[byteOffset / 4];
      return vec4u(v & 0xFFFF, v >> 16, 123, 123)
    `
  },
  rg16sint: {
    storageType: 'i32',
    texelType: 'vec4i',
    unpackWGSL: `
      let v = src[byteOffset / 4];
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
        unpack2x16float(src[byteOffset / 4]),
        unpack2x16float(src[byteOffset / 4 + 1]))
    `
  },
  rgba16uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
      let v0 = src[byteOffset / 4];
      let v1 = src[byteOffset / 4 + 1];
      return vec4u(v0 & 0xFFFF, v0 >> 16, v1 & 0xFFFF, v1 >> 16)
    `
  },
  rgba16sint: {
    storageType: 'i32',
    texelType: 'vec4i',
    unpackWGSL: `
      let v0 = src[byteOffset / 4];
      let v1 = src[byteOffset / 4 + 1];
      return vec4i(
        extractBits(v0, 0, 16),
        extractBits(v0, 16, 16),
        extractBits(v1, 0, 16),
        extractBits(v1, 16, 16),
      )
    `
  },
  r32float: {
    storageType: 'f32',
    texelType: 'vec4f',
    unpackWGSL: 'return vec4f(src[byteOffset / 4], 0.123, 0.123, 0.123)'
  },
  rgb10a2uint: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
      let v = src[byteOffset / 4];
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
      let v = src[byteOffset / 4];
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
      let v = unpack2x16unorm(src[byteOffset / 4])[byteOffset % 4 / 2];
      return vec4f(v, 0.123, 0.123, 0.123)
    `,
    useFragDepth: true
  },
  depth32float: {
    storageType: 'f32',
    texelType: 'vec4f',
    unpackWGSL: `
      let v = src[byteOffset / 4];
      return vec4f(v, 0.123, 0.123, 0.123)
    `,
    useFragDepth: true
  },
  stencil8: {
    storageType: 'u32',
    texelType: 'vec4u',
    unpackWGSL: `
      return vec4u(unpack4xU8(src[byteOffset / 4])[byteOffset % 4], 123, 123, 123)
    `,
    discardWithStencil: true
  }
};

function getCopyBufferToTextureViaRenderCode(format) {
  const info = kLoadValueFromStorageInfo[format];
  assert(!!info);
  const { storageType, texelType, unpackWGSL, useFragDepth, discardWithStencil } = info;

  const [depthDecl, depthCode] = useFragDepth ?
  ['@builtin(frag_depth) d: f32,', 'fs.d = fs.v[0];'] :
  ['', ''];

  const stencilCode = discardWithStencil ? 'if ((fs.v.r & vin.stencilMask) == 0) { discard; }' : '';

  return `
    struct Uniforms {
      numTexelRows: u32,
      bytesPerRow: u32,
      bytesPerSample: u32,
      sampleCount: u32,
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
    @group(0) @binding(1) var<storage> src: array<${storageType}>;

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
        coord.y * uni.bytesPerRow +
        (coord.x * uni.sampleCount + vin.sampleIndex) * uni.bytesPerSample;
      var fs: FSOutput;
      fs.v = unpack(byteOffset);
      ${depthCode}
      ${stencilCode}
      return fs;
    }
    `;
}

const s_copyBufferToTextureViaRenderPipelines = new WeakMap(


);

function copyBufferToTextureViaRender(
t,
encoder,
source,
dest,
size)
{
  const { format, sampleCount } = dest.texture;
  const origin = reifyOrigin3D(dest.origin ?? [0]);
  const copySize = reifyExtent3D(size);

  const msInfo = kLoadValueFromStorageInfo[format];
  assert(!!msInfo);
  const { useFragDepth, discardWithStencil } = msInfo;

  const { device } = t;
  const numBlits = discardWithStencil ? 8 : 1;
  for (let blitCount = 0; blitCount < numBlits; ++blitCount) {
    const code = getCopyBufferToTextureViaRenderCode(format);
    const stencilWriteMask = 1 << blitCount;
    const id = JSON.stringify({
      format,
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
        label: `blitCopyFor-${format}`,
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
            format,
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
            format
          }
        } :
        {
          fragment: {
            module,
            targets: [{ format }]
          }
        }),
        primitive: {
          topology: 'triangle-strip'
        },
        ...(sampleCount > 1 && { multisample: { count: sampleCount } })
      });
      pipelines.set(id, pipeline);
    }

    const info = kTextureFormatInfo[format];
    const uniforms = new Uint32Array([
    copySize.height, //  numTexelRows: u32,
    source.bytesPerRow, //  bytesPerRow: u32,
    info.bytesPerBlock, //  bytesPerSample: u32,
    dest.texture.sampleCount //  sampleCount: u32,
    ]);
    const uniformBuffer = t.makeBufferWithContents(
      uniforms,
      GPUBufferUsage.COPY_DST | GPUBufferUsage.UNIFORM
    );
    const storageBuffer = t.createBufferTracked({
      size: source.buffer.size,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
    });
    encoder.copyBufferToBuffer(source.buffer, 0, storageBuffer, 0, storageBuffer.size);
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
            depthStoreOp: 'store'
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

      const offset =
      (source.offset ?? 0) + (source.bytesPerRow ?? 0) * (source.rowsPerImage ?? 0) * l;
      const bindGroup = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
        { binding: 0, resource: { buffer: uniformBuffer } },
        { binding: 1, resource: { buffer: storageBuffer, offset } }]

      });

      pass.setBindGroup(0, bindGroup);
      pass.setStencilReference(0xff);
      pass.draw(4 * copySize.height * dest.texture.sampleCount, 1, 0, blitCount);
      pass.end();
    }
  }
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
  const format = texelViews[0].format;
  const { width, height, depthOrArrayLayers } = reifyExtent3D(desc.size);

  // Create the texture and then initialize each mipmap level separately.
  const texture = t.createTextureTracked({
    ...desc,
    format,
    usage: desc.usage | GPUTextureUsage.COPY_DST,
    mipLevelCount: texelViews.length
  });

  // Copy the texel view into each mip level layer.
  const commandEncoder = t.device.createCommandEncoder();
  const stagingBuffers = [];
  for (let mipLevel = 0; mipLevel < texelViews.length; mipLevel++) {
    const {
      bytesPerRow,
      rowsPerImage,
      mipSize: [mipWidth, mipHeight, mipDepthOrArray]
    } = getTextureCopyLayout(format, desc.dimension ?? '2d', [width, height, depthOrArrayLayers], {
      mipLevel
    });

    // Create a staging buffer to upload the texture mip level contents.
    const stagingBuffer = t.createBufferTracked({
      mappedAtCreation: true,
      size: bytesPerRow * mipHeight * mipDepthOrArray,
      usage: GPUBufferUsage.COPY_SRC
    });
    stagingBuffers.push(stagingBuffer);

    // Write the texels into the staging buffer.
    texelViews[mipLevel].writeTextureData(new Uint8Array(stagingBuffer.getMappedRange()), {
      bytesPerRow,
      rowsPerImage: mipHeight,
      subrectOrigin: [0, 0, 0],
      subrectSize: [mipWidth, mipHeight, mipDepthOrArray],
      sampleCount: texture.sampleCount
    });
    stagingBuffer.unmap();

    if (texture.sampleCount > 1 || isDepthOrStencilTextureFormat(format)) {
      copyBufferToTextureViaRender(
        t,
        commandEncoder,
        { buffer: stagingBuffer, bytesPerRow, rowsPerImage },
        { texture, mipLevel },
        [mipWidth, mipHeight, mipDepthOrArray]
      );
    } else {
      // Copy from the staging buffer into the texture.
      commandEncoder.copyBufferToTexture(
        { buffer: stagingBuffer, bytesPerRow, rowsPerImage },
        { texture, mipLevel },
        [mipWidth, mipHeight, mipDepthOrArray]
      );
    }
  }
  t.device.queue.submit([commandEncoder.finish()]);

  // Cleanup the staging buffers.
  stagingBuffers.forEach((value) => value.destroy());

  return texture;
}