/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, memcpy } from '../common/util/util.js';import {


  getBlockInfoForColorTextureFormat,
  getBlockInfoForTextureFormat,
  kEncodableTextureFormats,
  resolvePerAspectFormat } from
'./format_info.js';

import { checkElementsEqual } from './util/check_contents.js';
import { align } from './util/math.js';
import { physicalMipSizeFromTexture, virtualMipSize } from './util/texture/base.js';
import {
  bytesInACompleteRow,
  getTextureCopyLayout } from

'./util/texture/layout.js';

import { TexelView } from './util/texture/texel_view.js';
import {



  textureContentIsOKByT2B } from
'./util/texture/texture_ok.js';
import { createTextureFromTexelViews } from './util/texture.js';
import { reifyOrigin3D } from './util/unions.js';







const s_deviceToResourcesMap = new WeakMap();
/**
 * Gets a (cached) pipeline to render a texture to an rgba8unorm texture
 */
function getPipelineToRenderTextureToRGB8UnormTexture(device, texture) {
  if (!s_deviceToResourcesMap.has(device)) {
    s_deviceToResourcesMap.set(device, {
      pipelineByPipelineType: new Map()
    });
  }

  const { pipelineByPipelineType } = s_deviceToResourcesMap.get(device);
  const pipelineType =
  texture.dimension === '3d' ? '3d' : texture.depthOrArrayLayers > 1 ? '2d-array' : '2d';
  if (!pipelineByPipelineType.get(pipelineType)) {
    const [textureType, coordCode] =
    pipelineType === '3d' ?
    [
    'texture_3d',
    'vec3f(fsInput.texcoord, (f32(uni.baseArrayLayer) + 0.5) / f32(textureDimensions(ourTexture, 0).z))'] :

    pipelineType === '2d' ?
    ['texture_2d', 'fsInput.texcoord'] :
    ['texture_2d_array', 'fsInput.texcoord, uni.baseArrayLayer'];
    const code = `
      struct VSOutput {
        @builtin(position) position: vec4f,
        @location(0) texcoord: vec2f,
      };

      struct Uniforms {
        baseArrayLayer: u32,
      };

      @vertex fn vs(
        @builtin(vertex_index) vertexIndex : u32
      ) -> VSOutput {
          let pos = array(
             vec2f(-1, -1),
             vec2f(-1,  3),
             vec2f( 3, -1),
          );

          var vsOutput: VSOutput;

          let xy = pos[vertexIndex];

          vsOutput.position = vec4f(xy, 0.0, 1.0);
          vsOutput.texcoord = xy * vec2f(0.5, -0.5) + vec2f(0.5);

          return vsOutput;
       }

       @group(0) @binding(0) var ourSampler: sampler;
       @group(0) @binding(1) var ourTexture: ${textureType}<f32>;
       @group(0) @binding(2) var<uniform> uni: Uniforms;

       @fragment fn fs(fsInput: VSOutput) -> @location(0) vec4f {
          _ = uni;
          return textureSample(ourTexture, ourSampler, ${coordCode});
       }
    `;
    const module = device.createShaderModule({ code });
    const pipeline = device.createRenderPipeline({
      label: `layer rendered for ${pipelineType}`,
      layout: 'auto',
      vertex: {
        module,
        entryPoint: 'vs'
      },
      fragment: {
        module,
        entryPoint: 'fs',
        targets: [{ format: 'rgba8unorm' }]
      }
    });
    pipelineByPipelineType.set(pipelineType, pipeline);
  }
  const pipeline = pipelineByPipelineType.get(pipelineType);
  return { pipelineType, pipeline };
}







/**
 * Creates a 1 mip level texture with the contents of a TexelView.
 */
export function createTextureFromTexelView(
t,
texelView,
desc)
{
  return createTextureFromTexelViews(t, [texelView], desc);
}

export function createTextureFromTexelViewsMultipleMipmaps(
t,
texelViews,
desc)
{
  return createTextureFromTexelViews(t, texelViews, desc);
}

export function expectTexelViewComparisonIsOkInTexture(
t,
src,
exp,
size,
comparisonOptions = {
  maxIntDiff: 0,
  maxDiffULPsForNormFormat: 1,
  maxDiffULPsForFloatFormat: 1
})
{
  t.eventualExpectOK(
    textureContentIsOKByT2B(t, src, size, { expTexelView: exp }, comparisonOptions)
  );
}

export function expectSinglePixelComparisonsAreOkInTexture(
t,
src,
exp,
comparisonOptions = {
  maxIntDiff: 0,
  maxDiffULPsForNormFormat: 1,
  maxDiffULPsForFloatFormat: 1
})
{
  assert(exp.length > 0, 'must specify at least one pixel comparison');
  assert(
    kEncodableTextureFormats.includes(src.texture.format),
    () => `${src.texture.format} is not an encodable format`
  );
  const lowerCorner = [src.texture.width, src.texture.height, src.texture.depthOrArrayLayers];
  const upperCorner = [0, 0, 0];
  const expMap = new Map();
  const coords = [];
  for (const e of exp) {
    const coord = reifyOrigin3D(e.coord);
    const coordKey = JSON.stringify(coord);
    coords.push(coord);

    // Compute the minimum sub-rect that encompasses all the pixel comparisons. The
    // `lowerCorner` will become the origin, and the `upperCorner` will be used to compute the
    // size.
    lowerCorner[0] = Math.min(lowerCorner[0], coord.x);
    lowerCorner[1] = Math.min(lowerCorner[1], coord.y);
    lowerCorner[2] = Math.min(lowerCorner[2], coord.z);
    upperCorner[0] = Math.max(upperCorner[0], coord.x);
    upperCorner[1] = Math.max(upperCorner[1], coord.y);
    upperCorner[2] = Math.max(upperCorner[2], coord.z);

    // Build a sparse map of the coordinates to the expected colors for the texel view.
    assert(
      !expMap.has(coordKey),
      () => `duplicate pixel expectation at coordinate (${coord.x},${coord.y},${coord.z})`
    );
    expMap.set(coordKey, e.exp);
  }
  const size = [
  upperCorner[0] - lowerCorner[0] + 1,
  upperCorner[1] - lowerCorner[1] + 1,
  upperCorner[2] - lowerCorner[2] + 1];

  let expTexelView;
  if (Symbol.iterator in exp[0].exp) {
    expTexelView = TexelView.fromTexelsAsBytes(
      src.texture.format,
      (coord) => {
        const res = expMap.get(JSON.stringify(coord));
        assert(
          res !== undefined,
          () => `invalid coordinate (${coord.x},${coord.y},${coord.z}) in sparse texel view`
        );
        return res;
      }
    );
  } else {
    expTexelView = TexelView.fromTexelsAsColors(
      src.texture.format,
      (coord) => {
        const res = expMap.get(JSON.stringify(coord));
        assert(
          res !== undefined,
          () => `invalid coordinate (${coord.x},${coord.y},${coord.z}) in sparse texel view`
        );
        return res;
      }
    );
  }
  const coordsF = function* () {
    for (const coord of coords) {
      yield coord;
    }
  }();

  t.eventualExpectOK(
    textureContentIsOKByT2B(
      t,
      { ...src, origin: reifyOrigin3D(lowerCorner) },
      size,
      { expTexelView },
      comparisonOptions,
      coordsF
    )
  );
}

export function expectTexturesToMatchByRendering(
t,
actualTexture,
expectedTexture,
mipLevel,
origin,
size)
{
  // Render every layer of both textures at mipLevel to an rgba8unorm texture
  // that matches the size of the mipLevel. After each render, copy the
  // result to a buffer and expect the results from both textures to match.
  const { pipelineType, pipeline } = getPipelineToRenderTextureToRGB8UnormTexture(
    t.device,
    actualTexture
  );
  const readbackPromisesPerTexturePerLayer = [actualTexture, expectedTexture].map(
    (texture, ndx) => {
      const attachmentSize = virtualMipSize(
        actualTexture.dimension,
        [texture.width, texture.height, 1],
        mipLevel
      );
      const attachment = t.createTextureTracked({
        label: `readback${ndx}`,
        size: attachmentSize,
        format: 'rgba8unorm',
        usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
      });

      const sampler = t.device.createSampler();

      const numLayers = texture.depthOrArrayLayers;
      const readbackPromisesPerLayer = [];

      const uniformBuffer = t.createBufferTracked({
        label: 'expectTexturesToMatchByRendering:uniformBuffer',
        size: 4,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
      });

      for (let layer = 0; layer < numLayers; ++layer) {
        const viewDescriptor = {
          baseMipLevel: mipLevel,
          mipLevelCount: 1,
          dimension: pipelineType
        };

        const bindGroup = t.device.createBindGroup({
          layout: pipeline.getBindGroupLayout(0),
          entries: [
          { binding: 0, resource: sampler },
          {
            binding: 1,
            resource: texture.createView(viewDescriptor)
          },
          {
            binding: 2,
            resource: { buffer: uniformBuffer }
          }]

        });

        t.device.queue.writeBuffer(uniformBuffer, 0, new Uint32Array([layer]));

        const encoder = t.device.createCommandEncoder({
          label: 'expectTexturesToMatchByRendering'
        });
        const pass = encoder.beginRenderPass({
          colorAttachments: [
          {
            view: attachment.createView(),
            clearValue: [0.5, 0.5, 0.5, 0.5],
            loadOp: 'clear',
            storeOp: 'store'
          }]

        });
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bindGroup);
        pass.draw(3);
        pass.end();
        t.queue.submit([encoder.finish()]);

        const buffer = copyWholeTextureToNewBufferSimple(t, attachment, 0);

        readbackPromisesPerLayer.push(
          t.readGPUBufferRangeTyped(buffer, {
            type: Uint8Array,
            typedLength: buffer.size
          })
        );
      }
      return readbackPromisesPerLayer;
    }
  );

  t.eventualAsyncExpectation(async (niceStack) => {
    const readbacksPerTexturePerLayer = [];

    // Wait for all buffers to be ready
    for (const readbackPromises of readbackPromisesPerTexturePerLayer) {
      readbacksPerTexturePerLayer.push(await Promise.all(readbackPromises));
    }

    function arrayNotAllTheSameValue(arr, msg) {
      const first = arr[0];
      return arr.length <= 1 || arr.findIndex((v) => v !== first) >= 0 ?
      undefined :
      Error(`array is entirely ${first} so likely nothing was tested: ${msg || ''}`);
    }

    // Compare each layer of each texture as read from buffer.
    const [actualReadbacksPerLayer, expectedReadbacksPerLayer] = readbacksPerTexturePerLayer;
    for (let layer = 0; layer < actualReadbacksPerLayer.length; ++layer) {
      const actualReadback = actualReadbacksPerLayer[layer];
      const expectedReadback = expectedReadbacksPerLayer[layer];
      const sameOk =
      size.width === 0 ||
      size.height === 0 ||
      layer < origin.z ||
      layer >= origin.z + size.depthOrArrayLayers;
      t.expectOK(
        sameOk ? undefined : arrayNotAllTheSameValue(actualReadback.data, 'actualTexture')
      );
      t.expectOK(
        sameOk ? undefined : arrayNotAllTheSameValue(expectedReadback.data, 'expectedTexture')
      );
      t.expectOK(checkElementsEqual(actualReadback.data, expectedReadback.data), {
        mode: 'fail',
        niceStack
      });
      actualReadback.cleanup();
      expectedReadback.cleanup();
    }
  });
}

/**
 * Expect an entire GPUTexture to have a single color at the given mip level (defaults to 0).
 * MAINTENANCE_TODO: Remove this and/or replace it with a helper in TextureTestMixin.
 */
export function expectSingleColorWithTolerance(
t,
src,
format,
{
  size,
  exp,
  dimension = '2d',
  slice = 0,
  layout,
  maxFractionalDiff







})
{
  assert(slice === 0 || dimension === '2d', 'texture slices are only implemented for 2d textures');

  format = resolvePerAspectFormat(format, layout?.aspect);
  const { mipSize } = getTextureCopyLayout(format, dimension, size, layout);
  // MAINTENANCE_TODO: getTextureCopyLayout does not return the proper size for array textures,
  // i.e. it will leave the z/depth value as is instead of making it 1 when dealing with 2d
  // texture arrays. Since we are passing in the dimension, we should update it to return the
  // corrected size.
  const copySize = [
  mipSize[0],
  dimension !== '1d' ? mipSize[1] : 1,
  dimension === '3d' ? mipSize[2] : 1];


  // Create a TexelView that returns exp for all texels.
  const expTexelView = TexelView.fromTexelsAsColors(format, () => exp);
  const source = {
    texture: src,
    mipLevel: layout?.mipLevel ?? 0,
    aspect: layout?.aspect ?? 'all',
    origin: [0, 0, slice]
  };
  const comparisonOptions = {
    maxFractionalDiff: maxFractionalDiff ?? 0
  };
  t.eventualExpectOK(
    textureContentIsOKByT2B(t, source, copySize, { expTexelView }, comparisonOptions)
  );
}

export function copyWholeTextureToNewBufferSimple(
t,
texture,
mipLevel)
{
  const { blockWidth, blockHeight, bytesPerBlock } = getBlockInfoForTextureFormat(texture.format);
  const mipSize = physicalMipSizeFromTexture(texture, mipLevel);
  assert(bytesPerBlock !== undefined);

  const blocksPerRow = mipSize[0] / blockWidth;
  const blocksPerColumn = mipSize[1] / blockHeight;

  assert(blocksPerRow % 1 === 0);
  assert(blocksPerColumn % 1 === 0);

  const bytesPerRow = align(blocksPerRow * bytesPerBlock, 256);
  const byteLength = bytesPerRow * blocksPerColumn * mipSize[2];

  return copyWholeTextureToNewBuffer(
    t,
    { texture, mipLevel },
    {
      bytesPerBlock,
      bytesPerRow,
      rowsPerImage: blocksPerColumn,
      byteLength
    }
  );
}

export function copyWholeTextureToNewBuffer(
t,
{ texture, mipLevel },
resultDataLayout)





{
  const { byteLength, bytesPerRow, rowsPerImage } = resultDataLayout;
  const buffer = t.createBufferTracked({
    label: 'copyWholeTextureToNewBuffer:buffer',
    size: align(byteLength, 4), // this is necessary because we need to copy and map data from this buffer
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const mipSize = physicalMipSizeFromTexture(texture, mipLevel || 0);
  const encoder = t.device.createCommandEncoder({ label: 'copyWholeTextureToNewBuffer' });
  encoder.copyTextureToBuffer(
    { texture, mipLevel },
    { buffer, bytesPerRow, rowsPerImage },
    mipSize
  );
  t.device.queue.submit([encoder.finish()]);

  return buffer;
}

export function updateLinearTextureDataSubBox(
t,
format,
copySize,
copyParams)



{
  const { src, dest } = copyParams;
  const rowLength = bytesInACompleteRow(copySize.width, format);
  for (const texel of iterateBlockRows(copySize, format)) {
    const srcOffsetElements = getTexelOffsetInBytes(src.dataLayout, format, texel, src.origin);
    const dstOffsetElements = getTexelOffsetInBytes(dest.dataLayout, format, texel, dest.origin);
    memcpy(
      { src: src.data, start: srcOffsetElements, length: rowLength },
      { dst: dest.data, start: dstOffsetElements }
    );
  }
}

/** Offset for a particular texel in the linear texture data */
export function getTexelOffsetInBytes(
textureDataLayout,
format,
texel,
origin = { x: 0, y: 0, z: 0 })
{
  const { offset, bytesPerRow, rowsPerImage } = textureDataLayout;
  const info = getBlockInfoForColorTextureFormat(format);

  assert(texel.x % info.blockWidth === 0);
  assert(texel.y % info.blockHeight === 0);
  assert(origin.x % info.blockWidth === 0);
  assert(origin.y % info.blockHeight === 0);

  const bytesPerImage = rowsPerImage * bytesPerRow;

  return (
    offset +
    (texel.z + origin.z) * bytesPerImage +
    (texel.y + origin.y) / info.blockHeight * bytesPerRow +
    (texel.x + origin.x) / info.blockWidth * info.bytesPerBlock);

}

export function* iterateBlockRows(
size,
format)
{
  if (size.width === 0 || size.height === 0 || size.depthOrArrayLayers === 0) {
    // do not iterate anything for an empty region
    return;
  }
  const info = getBlockInfoForTextureFormat(format);
  assert(size.height % info.blockHeight === 0);
  // Note: it's important that the order is in increasing memory address order.
  for (let z = 0; z < size.depthOrArrayLayers; ++z) {
    for (let y = 0; y < size.height; y += info.blockHeight) {
      yield {
        x: 0,
        y,
        z
      };
    }
  }
}