/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Test fragment shader builtin variables and inter-stage variables

* test builtin(position)
* test @interpolate
* test builtin(sample_index)
* test builtin(front_facing)
* test builtin(sample_mask)

Note: @interpolate settings and sample_index affect whether or not the fragment shader
is evaluated per-fragment or per-sample. With @interpolate(, sample) or usage of
@builtin(sample_index) the fragment shader should be executed per-sample.

* sample_mask output is tested in
  src/webgpu/api/operation/render_pipeline/sample_mask.spec.ts

* frag_depth output is tested in
  src/webgpu/api/operation/rendering/depth_clip_clamp.spec.ts
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ErrorWithExtra, assert, range, unreachable } from '../../../../common/util/util.js';

import { getBlockInfoForTextureFormat } from '../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';
import { getProvokingVertexForFlatInterpolationEitherSampling } from '../../../inter_stage.js';
import { getMultisampleFragmentOffsets } from '../../../multisample_info.js';
import * as ttu from '../../../texture_test_utils.js';
import { dotProduct, subtractVectors, align } from '../../../util/math.js';

import { TexelView } from '../../../util/texture/texel_view.js';
import { findFailedPixels } from '../../../util/texture/texture_ok.js';

class FragmentBuiltinTest extends AllFeaturesMaxLimitsGPUTest {}

export const g = makeTestGroup(FragmentBuiltinTest);

const s_deviceToPipelineMap = new WeakMap(





);

/**
 * Returns an object of pipelines associated
 * by weakmap to a device so we can cache pipelines.
 */
function getPipelinesForDevice(device) {
  let pipelines = s_deviceToPipelineMap.get(device);
  if (!pipelines) {
    pipelines = {};
    s_deviceToPipelineMap.set(device, pipelines);
  }
  return pipelines;
}

/**
 * Gets a compute pipeline that will copy the given texture if passed
 * a dispatch size of texture.width, texture.height
 * @param device a device
 * @param texture texture the pipeline is needed for.
 * @returns A GPUComputePipeline
 */
function getCopyMultisamplePipelineForDevice(device, textures) {
  assert(textures.length === 4);
  assert(textures[0].sampleCount === textures[1].sampleCount);
  assert(textures[0].sampleCount === textures[2].sampleCount);
  assert(textures[0].sampleCount === textures[3].sampleCount);

  const pipelineType = textures[0].sampleCount > 1 ? 'texture_multisampled_2d' : 'texture_2d';
  const pipelines = getPipelinesForDevice(device);
  let pipeline = pipelines[pipelineType];
  if (!pipeline) {
    const isMultisampled = pipelineType === 'texture_multisampled_2d';
    const numSamples = isMultisampled ? 'textureNumSamples(texture0)' : '1u';
    const sampleIndex = isMultisampled ? 'sampleIndex' : '0';
    const module = device.createShaderModule({
      code: `
        @group(0) @binding(0) var texture0: ${pipelineType}<f32>;
        @group(0) @binding(1) var texture1: ${pipelineType}<f32>;
        @group(0) @binding(2) var texture2: ${pipelineType}<f32>;
        @group(0) @binding(3) var texture3: ${pipelineType}<f32>;
        @group(0) @binding(4) var<storage, read_write> buffer: array<f32>;

        @compute @workgroup_size(1) fn cs(@builtin(global_invocation_id) id: vec3u) {
          let numSamples = ${numSamples};
          let dimensions = textureDimensions(texture0);
          let sampleIndex = id.x % numSamples;
          let tx = id.x / numSamples;
          let offset = ((id.y * dimensions.x + tx) * numSamples + sampleIndex) * 4;
          let r = vec4u(textureLoad(texture0, vec2u(tx, id.y), ${sampleIndex}) * 255.0);
          let g = vec4u(textureLoad(texture1, vec2u(tx, id.y), ${sampleIndex}) * 255.0);
          let b = vec4u(textureLoad(texture2, vec2u(tx, id.y), ${sampleIndex}) * 255.0);
          let a = vec4u(textureLoad(texture3, vec2u(tx, id.y), ${sampleIndex}) * 255.0);

          // expand rgba8unorm values back to their byte form, add them together
          // and cast them to an f32 so we can recover the f32 values we encoded
          // in the rgba8unorm texture.
          buffer[offset + 0] = bitcast<f32>(dot(r, vec4u(0x1000000, 0x10000, 0x100, 0x1)));
          buffer[offset + 1] = bitcast<f32>(dot(g, vec4u(0x1000000, 0x10000, 0x100, 0x1)));
          buffer[offset + 2] = bitcast<f32>(dot(b, vec4u(0x1000000, 0x10000, 0x100, 0x1)));
          buffer[offset + 3] = bitcast<f32>(dot(a, vec4u(0x1000000, 0x10000, 0x100, 0x1)));
        }
      `
    });

    pipeline = device.createComputePipeline({
      label: 'copy multisampled texture pipeline',
      layout: 'auto',
      compute: {
        module,
        entryPoint: 'cs'
      }
    });

    pipelines[pipelineType] = pipeline;
  }
  return pipeline;
}

function isTextureSameDimensions(a, b) {
  return (
    a.sampleCount === b.sampleCount &&
    a.width === b.width &&
    a.height === b.height &&
    a.depthOrArrayLayers === b.depthOrArrayLayers);

}

/**
 * Copies a texture (even if multisampled) to a buffer
 * @param t a gpu test
 * @param texture texture to copy
 * @returns buffer with copy of texture, mip level 0, array layer 0.
 */
function copyRGBA8EncodedFloatTexturesToBufferIncludingMultisampledTextures(
t,
textures)
{
  assert(textures.length === 4);
  assert(isTextureSameDimensions(textures[0], textures[1]));
  assert(isTextureSameDimensions(textures[0], textures[2]));
  assert(isTextureSameDimensions(textures[0], textures[3]));
  const { width, height, sampleCount } = textures[0];

  const copyBuffer = t.createBufferTracked({
    size: width * height * sampleCount * 4 * 4,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  const buffer = t.createBufferTracked({
    size: copyBuffer.size,
    usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST
  });

  const pipeline = getCopyMultisamplePipelineForDevice(t.device, textures);
  const encoder = t.device.createCommandEncoder();

  const textureEntries = textures.map(
    (texture, i) => ({ binding: i, resource: texture.createView() })
  );

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [...textureEntries, { binding: 4, resource: { buffer: copyBuffer } }]
  });

  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(width * sampleCount, height);
  pass.end();

  encoder.copyBufferToBuffer(copyBuffer, 0, buffer, 0, buffer.size);

  t.device.queue.submit([encoder.finish()]);

  return buffer;
}

/* column constants */
const kX = 0;
const kY = 1;
const kZ = 2;
const kW = 3;

/**
 * Gets a column of values from an array of arrays.
 */
function getColumn(values, colNum) {
  return values.map((v) => v[colNum]);
}

/**
 * Computes the linear interpolation of 3 values from 3 vertices of a triangle
 * based on barycentric coordinates
 */
function linearInterpolation(baryCoords, interCoords) {
  return dotProduct(baryCoords, interCoords);
}

/**
 * Computes the perspective interpolation of 3 values from 3 vertices of a
 * triangle based on barycentric coordinates and their corresponding clip space
 * W coordinates.
 */
function perspectiveInterpolation(
barycentricCoords,
clipSpaceTriangleCoords,
interCoords)
{
  const [a, b, c] = barycentricCoords;
  const [fa, fb, fc] = interCoords;
  const wa = clipSpaceTriangleCoords[0][kW];
  const wb = clipSpaceTriangleCoords[1][kW];
  const wc = clipSpaceTriangleCoords[2][kW];

  return (a * fa / wa + b * fb / wb + c * fc / wc) / (a / wa + b / wb + c / wc);
}

/**
 * Converts clip space coords to NDC coords
 */
function clipSpaceToNDC(point) {
  return point.map((v) => v / point[kW]);
}

/**
 * Converts NDC coords to window coords.
 */
function ndcToWindow(ndcPoint, viewport) {
  const [xd, yd, zd] = ndcPoint;
  const px = viewport[2];
  const py = viewport[3];
  const ox = viewport[0] + px / 2;
  const oy = viewport[1] + py / 2;
  const zNear = viewport[4];
  const zFar = viewport[5];

  return [
  px / 2 * xd + ox,
  -py / 2 * yd + oy,
  zd * (zFar - zNear) + zNear];

}

/**
 * Computes barycentric coordinates of triangle for point p.
 * @param trianglePoints points for triangle
 * @param p point in triangle (or relative to it)
 * @returns barycentric coords of p
 */
function calcBarycentricCoordinates(trianglePoints, p) {
  const [a, b, c] = trianglePoints;

  const v0 = subtractVectors(b, a);
  const v1 = subtractVectors(c, a);
  const v2 = subtractVectors(p, a);

  const dot00 = dotProduct(v0, v0);
  const dot01 = dotProduct(v0, v1);
  const dot11 = dotProduct(v1, v1);
  const dot20 = dotProduct(v2, v0);
  const dot21 = dotProduct(v2, v1);

  const denom = 1 / (dot00 * dot11 - dot01 * dot01);
  const v = (dot11 * dot20 - dot01 * dot21) * denom;
  const w = (dot00 * dot21 - dot01 * dot20) * denom;
  const u = 1 - v - w;

  return [u, v, w];
}

/**
 * Returns true if point is inside triangle
 */
function isInsideTriangle(barycentricCoords) {
  for (const v of barycentricCoords) {
    if (v < 0 || v > 1) {
      return false;
    }
  }
  return true;
}

/**
 * Returns true if windowPoints define a clockwise triangle
 */
function isTriangleClockwise(windowPoints) {
  let sum = 0;
  for (let i = 0; i < 3; ++i) {
    const p0 = windowPoints[i];
    const p1 = windowPoints[(i + 1) % 3];
    sum += p0[kX] * p1[kY] - p1[kX] * p0[kY];
  }
  return sum >= 0;
}














/**
 * For each sample in texture, computes the values that would be provided
 * to the shader as `@builtin(position)` if the texture was a render target
 * and every point in the texture was inside the triangle.
 * @param texture The texture
 * @param clipSpacePoints triangle points in clip space
 * @returns the expected values for each sample
 */
function generateFragmentInputs({
  width,
  height,
  nearFar,
  sampleCount,
  frontFace,
  clipSpacePoints,
  interpolateFn








}) {
  const expected = new Float32Array(width * height * sampleCount * 4);

  const viewport = [0, 0, width, height, ...nearFar];

  // For each triangle
  for (let vertexIndex = 0; vertexIndex < clipSpacePoints.length; vertexIndex += 3) {
    const ndcPoints = clipSpacePoints.slice(vertexIndex, vertexIndex + 3).map(clipSpaceToNDC);
    const windowPoints = ndcPoints.map((p) => ndcToWindow(p, viewport));
    const windowPoints2D = windowPoints.map((p) => p.slice(0, 2));

    const cw = isTriangleClockwise(windowPoints2D);
    const frontFacing = frontFace === 'cw' ? cw : !cw;
    const fragmentOffsets = getMultisampleFragmentOffsets(sampleCount);

    for (let y = 0; y < height; ++y) {
      for (let x = 0; x < width; ++x) {
        let sampleMask = 0;
        for (let sampleIndex = 0; sampleIndex < sampleCount; ++sampleIndex) {
          const localSampleMask = 1 << sampleIndex;
          const multisampleOffset = fragmentOffsets[sampleIndex];
          const sampleFragmentPoint = [x + multisampleOffset[0], y + multisampleOffset[1]];
          const sampleBarycentricCoords = calcBarycentricCoordinates(
            windowPoints2D,
            sampleFragmentPoint
          );

          const inside = isInsideTriangle(sampleBarycentricCoords);
          if (inside) {
            sampleMask |= localSampleMask;
          }
        }

        for (let sampleIndex = 0; sampleIndex < sampleCount; ++sampleIndex) {
          const fragmentPoint = [x + 0.5, y + 0.5];
          const multisampleOffset = fragmentOffsets[sampleIndex];
          const sampleFragmentPoint = [x + multisampleOffset[0], y + multisampleOffset[1]];
          const fragmentBarycentricCoords = calcBarycentricCoordinates(
            windowPoints2D,
            fragmentPoint
          );
          const sampleBarycentricCoords = calcBarycentricCoordinates(
            windowPoints2D,
            sampleFragmentPoint
          );

          const inside = isInsideTriangle(sampleBarycentricCoords);
          if (inside) {
            const output = interpolateFn({
              baseVertexIndex: vertexIndex,
              fragmentPoint,
              fragmentBarycentricCoords,
              sampleBarycentricCoords,
              clipSpacePoints,
              ndcPoints,
              windowPoints,
              sampleIndex,
              sampleMask,
              frontFacing
            });

            const offset = ((y * width + x) * sampleCount + sampleIndex) * 4;
            expected.set(output, offset);
          }
        }
      }
    }
  }
  return expected;
}

/**
 * Computes 'builtin(position)`
 */
function computeFragmentPosition({
  fragmentPoint,
  fragmentBarycentricCoords,
  clipSpacePoints,
  windowPoints
}) {
  return [
  fragmentPoint[0],
  fragmentPoint[1],
  linearInterpolation(fragmentBarycentricCoords, getColumn(windowPoints, kZ)),
  1 /
  perspectiveInterpolation(
    fragmentBarycentricCoords,
    clipSpacePoints,
    getColumn(clipSpacePoints, kW)
  )];

}

/**
 * Creates a function that will compute the interpolation of an inter-stage variable.
 */
async function createInterStageInterpolationFn(
t,
interStagePoints,
type,
sampling)
{
  const provokingVertex =
  type === 'flat' && sampling === 'either' ?
  await getProvokingVertexForFlatInterpolationEitherSampling(t) :
  'first';

  return function ({
    baseVertexIndex,
    fragmentBarycentricCoords,
    sampleBarycentricCoords,
    clipSpacePoints
  }) {
    const triangleInterStagePoints = interStagePoints.slice(baseVertexIndex, baseVertexIndex + 3);
    const barycentricCoords =
    sampling === 'center' || sampling === undefined ?
    fragmentBarycentricCoords :
    sampleBarycentricCoords;
    switch (type) {
      case 'perspective':
        return triangleInterStagePoints[0].map((_, colNum) =>
        perspectiveInterpolation(
          barycentricCoords,
          clipSpacePoints,
          getColumn(triangleInterStagePoints, colNum)
        )
        );
        break;
      case 'linear':
        return triangleInterStagePoints[0].map((_, colNum) =>
        linearInterpolation(barycentricCoords, getColumn(triangleInterStagePoints, colNum))
        );
        break;
      case 'flat':
        return triangleInterStagePoints[provokingVertex === 'first' ? 0 : 2];
        break;
      default:
        unreachable();
    }
  };
}

/**
 * Creates a function that will compute the interpolation of an inter-stage variable
 * and then return [1, 0, 0, 0] if all interpolated values are between 0.0 and 1.0 inclusive
 * or [-1, 0, 0, 0] otherwise.
 */
async function createInterStageInterpolationBetween0And1TestFn(
t,
interStagePoints,
type,
sampling)
{
  const interpolateFn = await createInterStageInterpolationFn(t, interStagePoints, type, sampling);
  return function (fragData) {
    const interpolatedValues = interpolateFn(fragData);
    const allTrue = interpolatedValues.reduce((all, v) => all && v >= 0 && v <= 1, true);
    return [allTrue ? 1 : -1, 0, 0, 0];
  };
}

/**
 * Computes 'builtin(sample_index)'
 */
function computeFragmentSampleIndex({ sampleIndex }) {
  return [sampleIndex, 0, 0, 0];
}

/**
 * Computes 'builtin(front_facing)'
 */
function computeFragmentFrontFacing({ frontFacing }) {
  return [frontFacing ? 1 : 0, 0, 0, 0];
}

/**
 * Computes 'builtin(sample_mask)'
 */
function computeSampleMask({ sampleMask }) {
  return [sampleMask, 0, 0, 0];
}

/**
 * Renders float32 fragment shader inputs values to 4 rgba8unorm textures that
 * can be multisampled textures. It stores each of the channels, r, g, b, a of
 * the shader input to a separate texture, doing the math required to store the
 * float32 value into an rgba8unorm texel.
 *
 * Note: We could try to store the output to an vec4f storage buffer.
 * Unfortunately, using a storage buffer has the issue that we need to compute
 * an index with the very thing we're trying to test. Similarly, if we used a
 * storage texture we would need to compute texture locations with the things
 * we're trying to test. Also, using a storage buffer seems to affect certain
 * backends like M1 Mac so it seems better to stick to rgba8unorm here and test
 * using a storage buffer in a fragment shader separately.
 *
 * We can't use rgba32float, nor rgba16float, nor rgba32uint as they can't be
 * multisampled in all feature levels.
 */
async function renderFragmentShaderInputsTo4TexturesAndReadbackValues(
t,
{
  interpolationType,
  interpolationSampling,
  sampleCount,
  width,
  height,
  nearFar,
  frontFace,
  clipSpacePoints,
  interStagePoints,
  fragInCode,
  outputCode












})
{
  const interpolate = `${interpolationType}${
  interpolationSampling ? `, ${interpolationSampling}` : ''
  }`;
  const module = t.device.createShaderModule({
    code: `
      struct Uniforms {
        resolution: vec2f,
      };

      @group(0) @binding(0) var<uniform> uni: Uniforms;

      struct VertexOut {
        @builtin(position) position: vec4f,
        @location(0) @interpolate(${interpolate}) interpolatedValue: vec4f,
      };

      @vertex fn vs(@builtin(vertex_index) vNdx: u32) -> VertexOut {
        let pos = array(
          ${clipSpacePoints.map((p) => `vec4f(${p.join(', ')})`).join(', ')}
        );
        let interStage = array(
          ${interStagePoints.map((p) => `vec4f(${p.join(', ')})`).join(', ')}
        );
        var v: VertexOut;
        v.position = pos[vNdx];
        v.interpolatedValue = interStage[vNdx];
        _ = uni;
        return v;
      }

      struct FragmentIn {
        @builtin(position) position: vec4f,
@location(0) @interpolate(${interpolate}) interpolatedValue: vec4f,
        ${fragInCode}
      };

      struct FragOut {
        @location(0) out0: vec4f,
        @location(1) out1: vec4f,
        @location(2) out2: vec4f,
        @location(3) out3: vec4f,
      };

      fn u32ToRGBAUnorm(u: u32) -> vec4f {
        return vec4f(
          f32((u >> 24) & 0xFF) / 255.0,
          f32((u >> 16) & 0xFF) / 255.0,
          f32((u >>  8) & 0xFF) / 255.0,
          f32((u >>  0) & 0xFF) / 255.0,
        );
      }

      @fragment fn fs(fin: FragmentIn) -> FragOut {
        var f: FragOut;
        let v = ${outputCode};
        let u = bitcast<vec4u>(v);
        f.out0 = u32ToRGBAUnorm(u[0]);
        f.out1 = u32ToRGBAUnorm(u[1]);
        f.out2 = u32ToRGBAUnorm(u[2]);
        f.out3 = u32ToRGBAUnorm(u[3]);
        _ = fin.interpolatedValue;
        return f;
      }
    `
  });

  const textures = range(4, () =>
  t.createTextureTracked({
    size: [width, height],
    usage:
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.TEXTURE_BINDING |
    GPUTextureUsage.COPY_SRC,
    format: 'rgba8unorm',
    sampleCount
  })
  );

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module,
      entryPoint: 'vs'
    },
    fragment: {
      module,
      entryPoint: 'fs',
      targets: textures.map(() => ({ format: 'rgba8unorm' }))
    },
    ...(frontFace && {
      primitive: {
        frontFace
      }
    }),
    multisample: {
      count: sampleCount
    }
  });

  const uniformBuffer = t.createBufferTracked({
    size: 8,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
  });
  t.device.queue.writeBuffer(uniformBuffer, 0, new Float32Array([width, height]));

  const viewport = [0, 0, width, height, ...nearFar];

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: { buffer: uniformBuffer } }]
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: textures.map((texture) => ({
      view: texture.createView(),
      loadOp: 'clear',
      storeOp: 'store'
    }))
  });
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.setViewport(viewport[0], viewport[1], viewport[2], viewport[3], viewport[4], viewport[5]);
  pass.draw(clipSpacePoints.length);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const buffer = copyRGBA8EncodedFloatTexturesToBufferIncludingMultisampledTextures(t, textures);
  await buffer.mapAsync(GPUMapMode.READ);
  return new Float32Array(buffer.getMappedRange());
}

function checkSampleRectsApproximatelyEqual({
  width,
  height,
  sampleCount,
  actual,
  expected,
  maxDiffULPsForFloatFormat







}) {
  const subrectOrigin = [0, 0, 0];
  const subrectSize = [width * sampleCount, height, 1];
  const areaDesc = {
    bytesPerRow: width * sampleCount * 4 * 4,
    rowsPerImage: height,
    subrectOrigin,
    subrectSize
  };

  const format = 'rgba32float';
  const actTexelView = TexelView.fromTextureDataByReference(
    format,
    new Uint8Array(actual.buffer),
    areaDesc
  );
  const expTexelView = TexelView.fromTextureDataByReference(
    format,
    new Uint8Array(expected.buffer),
    areaDesc
  );

  const failedPixelsMessage = findFailedPixels(
    format,
    { x: 0, y: 0, z: 0 },
    { width: width * sampleCount, height, depthOrArrayLayers: 1 },
    { actTexelView, expTexelView },
    { maxDiffULPsForFloatFormat }
  );

  if (failedPixelsMessage !== undefined) {
    const msg = 'Texture level had unexpected contents:\n' + failedPixelsMessage;
    return new ErrorWithExtra(msg, () => ({
      expTexelView,
      actTexelView
    }));
  }

  return undefined;
}

g.test('inputs,position').
desc(
  `
    Test fragment shader builtin(position) values.

    Note: @builtin(position) is always a fragment position, never a sample position.
  `
).
params((u) =>
u //
.combine('nearFar', [[0, 1], [0.25, 0.75]]).
combine('sampleCount', [1, 4]).
combine('interpolation', [
{ type: 'perspective', sampling: 'center' },
{ type: 'perspective', sampling: 'centroid' },
{ type: 'perspective', sampling: 'sample' },
{ type: 'linear', sampling: 'center' },
{ type: 'linear', sampling: 'centroid' },
{ type: 'linear', sampling: 'sample' },
{ type: 'flat', sampling: 'first' },
{ type: 'flat', sampling: 'either' }]
)
).
beforeAllSubcases((t) => {
  const {
    interpolation: { type, sampling }
  } = t.params;
  t.skipIfInterpolationTypeOrSamplingNotSupported({ type, sampling });
}).
fn(async (t) => {
  const {
    nearFar,
    sampleCount,
    interpolation: { type, sampling }
  } = t.params;

  const clipSpacePoints = [// ndc values
  [0.333, 0.333, 0.333, 0.333], //  1,  1, 1
  [1.0, -3.0, 0.25, 1.0], //  1, -3, 0.25
  [-1.5, 0.5, 0.25, 0.5] // -3,  1, 0.5
  ];

  const interStagePoints = [
  [1, 2, 3, 4],
  [5, 6, 7, 8],
  [9, 10, 11, 12]];


  const width = 4;
  const height = 4;
  const actual = await renderFragmentShaderInputsTo4TexturesAndReadbackValues(t, {
    interpolationType: type,
    interpolationSampling: sampling,
    sampleCount,
    width,
    height,
    nearFar,
    clipSpacePoints,
    interStagePoints,
    fragInCode: '',
    outputCode: 'fin.position'
  });

  const expected = generateFragmentInputs({
    width,
    height,
    nearFar,
    sampleCount,
    clipSpacePoints,
    interpolateFn: computeFragmentPosition
  });

  // Since @builtin(position) is always a fragment position, never a sample position, check
  // the first coordinate. It should be 0.5, 0.5 always. This is just to double check
  // that computeFragmentPosition is generating the correct values.
  assert(expected[0] === 0.5);
  assert(expected[1] === 0.5);

  t.expectOK(
    checkSampleRectsApproximatelyEqual({
      width,
      height,
      sampleCount,
      actual,
      expected,
      maxDiffULPsForFloatFormat: 4
    })
  );
});

g.test('inputs,interStage').
desc(
  `
    Test fragment shader inter-stage variable values except for centroid interpolation.
  `
).
params((u) =>
u //
.combine('nearFar', [[0, 1], [0.25, 0.75]]).
combine('sampleCount', [1, 4]).
combine('interpolation', [
{ type: 'perspective' },
{ type: 'perspective', sampling: 'center' },
{ type: 'perspective', sampling: 'sample' },
{ type: 'linear' },
{ type: 'linear', sampling: 'center' },
{ type: 'linear', sampling: 'sample' },
{ type: 'flat' },
{ type: 'flat', sampling: 'first' },
{ type: 'flat', sampling: 'either' }]
)
).
beforeAllSubcases((t) => {
  const {
    interpolation: { type, sampling }
  } = t.params;
  t.skipIfInterpolationTypeOrSamplingNotSupported({ type, sampling });
}).
fn(async (t) => {
  const {
    nearFar,
    sampleCount,
    interpolation: { type, sampling }
  } = t.params;

  const clipSpacePoints = [// ndc values
  [0.333, 0.333, 0.333, 0.333], //  1,  1, 1
  [1.0, -3.0, 0.25, 1.0], //  1, -3, 0.25
  [-1.5, 0.5, 0.25, 0.5] // -3,  1, 0.5
  ];

  const interStagePoints = [
  [1, 2, 3, 4],
  [5, 6, 7, 8],
  [9, 10, 11, 12]];


  const width = 4;
  const height = 4;
  const actual = await renderFragmentShaderInputsTo4TexturesAndReadbackValues(t, {
    interpolationType: type,
    interpolationSampling: sampling,
    sampleCount,
    width,
    height,
    nearFar,
    clipSpacePoints,
    interStagePoints,
    fragInCode: '',
    outputCode: 'fin.interpolatedValue'
  });

  const expected = generateFragmentInputs({
    width,
    height,
    nearFar,
    sampleCount,
    clipSpacePoints,
    interpolateFn: await createInterStageInterpolationFn(t, interStagePoints, type, sampling)
  });

  t.expectOK(
    checkSampleRectsApproximatelyEqual({
      width,
      height,
      sampleCount,
      actual,
      expected,
      maxDiffULPsForFloatFormat: 4
    })
  );
});

g.test('inputs,interStage,centroid').
desc(
  `
    Test fragment shader inter-stage variable values in centroid sampling mode.

    Centroid sampling mode is trying to solve the following issue

    +-------------+
    |....s1|/     |
    |......|      |
    |...../|   s2 |
    +------C------+
    |s3./  |      |
    |../   |      |
    |./    |s4    |
    +-------------+

    Above is a diagram of a texel where s1, s2, s3, s4 are sample points,
    C is the center of the texel and the diagonal line is some edge of
    a triangle. s1 and s3 are inside the triangle. In sampling = 'center'
    modes, the interpolated value will be relative to C. The problem is,
    C is outside of the triangle. In sample = 'centroid' mode, the
    interpolated value will be computed relative to some point inside the
    portion of the triangle inside the texel. While ideally it would be
    the actual centroid, the specs from the various APIs suggest the only
    guarantee is it's inside the triangle.

    So, we set the interStage values to barycentric coords. We expect
    that when sampling mode is 'center', some interpolated values
    will be outside of the triangle (ie, one or more of their values will
    be outside the 0 to 1 range). In sampling mode = 'centroid' mode, none
    of the values will be outside of the 0 to 1 range.

    Note: generateFragmentInputs below generates "expected". Values not
    rendered to will be 0. Values rendered to outside the triangle will
    be -1. Values rendered to inside the triangle will be 1. Manually
    checking, "expected" for sampling = 'center' should have a couple of
    -1 values where as "expected" for sampling = 'centroid' should not.
    This was verified with manual testing.

    Since we only care about inside vs outside of the triangle, having
    createInterStageInterpolationFn use the interpolated value relative
    to the sample point when sampling = 'centroid' will give us a value
    inside the triangle, which is good enough for our test.
  `
).
params((u) =>
u //
.combine('nearFar', [[0, 1], [0.25, 0.75]]).
combine('sampleCount', [1, 4]).
combine('interpolation', [
{ type: 'perspective', sampling: 'center' },
{ type: 'perspective', sampling: 'centroid' },
{ type: 'linear', sampling: 'center' },
{ type: 'linear', sampling: 'centroid' }]
)
).
beforeAllSubcases((t) => {
  const {
    interpolation: { type, sampling }
  } = t.params;
  t.skipIfInterpolationTypeOrSamplingNotSupported({ type, sampling });
}).
fn(async (t) => {
  const {
    nearFar,
    sampleCount,
    interpolation: { type, sampling }
  } = t.params;
  //
  // We're drawing 1 triangle that cut the viewport
  //
  //  -1   0   1
  //   +===+===+  2
  //   |\..|...|
  //   +---+---+  1  <---
  //   |  \|...|       |
  //   +---+---+  0    | viewport
  //   |   |\..|       |
  //   +---+---+ -1  <---
  //   |   |  \|
  //   +===+===+ -2


  const clipSpacePoints = [// ndc values
  [1, -2, 0, 1],
  [-1, 2, 0, 1],
  [1, 2, 0, 1]];



  const interStagePoints = [
  [1, 0, 0, 0],
  [0, 1, 0, 0],
  [0, 0, 1, 0]];


  const width = 4;
  const height = 4;
  const actual = await renderFragmentShaderInputsTo4TexturesAndReadbackValues(t, {
    interpolationType: type,
    interpolationSampling: sampling,
    sampleCount,
    width,
    height,
    nearFar,
    clipSpacePoints,
    interStagePoints,
    fragInCode: '',
    outputCode:
    'vec4f(select(-1.0, 1.0, all(fin.interpolatedValue >= vec4f(0)) && all(fin.interpolatedValue <= vec4f(1))), 0, 0, 0)'
  });

  const expected = generateFragmentInputs({
    width,
    height,
    nearFar,
    sampleCount,
    clipSpacePoints,
    interpolateFn: await createInterStageInterpolationBetween0And1TestFn(
      t,
      interStagePoints,
      type,
      sampling
    )
  });

  t.expectOK(
    checkSampleRectsApproximatelyEqual({
      width,
      height,
      sampleCount,
      actual,
      expected,
      maxDiffULPsForFloatFormat: 3
    })
  );
});

g.test('inputs,sample_index').
desc(
  `
    Test fragment shader builtin(sample_index) values.
  `
).
params((u) =>
u //
.combine('nearFar', [[0, 1], [0.25, 0.75]]).
combine('sampleCount', [1, 4]).
combine('interpolation', [
{ type: 'perspective', sampling: 'center' },
{ type: 'perspective', sampling: 'centroid' },
{ type: 'perspective', sampling: 'sample' },
{ type: 'linear', sampling: 'center' },
{ type: 'linear', sampling: 'centroid' },
{ type: 'linear', sampling: 'sample' },
{ type: 'flat', sampling: 'first' },
{ type: 'flat', sampling: 'either' }]
)
).
beforeAllSubcases((t) => {
  t.skipIf(t.isCompatibility, 'sample_index is not supported in compatibility mode');
}).
fn(async (t) => {
  const {
    nearFar,
    sampleCount,
    interpolation: { type, sampling }
  } = t.params;

  const clipSpacePoints = [// ndc values
  [0.333, 0.333, 0.333, 0.333], //  1,  1, 1
  [1.0, -3.0, 0.25, 1.0], //  1, -3, 0.25
  [-1.5, 0.5, 0.25, 0.5] // -3,  1, 0.5
  ];

  const interStagePoints = [
  [1, 2, 3, 4],
  [5, 6, 7, 8],
  [9, 10, 11, 12]];


  const width = 4;
  const height = 4;
  const actual = await renderFragmentShaderInputsTo4TexturesAndReadbackValues(t, {
    interpolationType: type,
    interpolationSampling: sampling,
    sampleCount,
    width,
    height,
    nearFar,
    clipSpacePoints,
    interStagePoints,
    fragInCode: `@builtin(sample_index) sampleIndex: u32,`,
    outputCode: 'vec4f(f32(fin.sampleIndex), 0, 0, 0)'
  });

  const expected = generateFragmentInputs({
    width,
    height,
    nearFar,
    sampleCount,
    clipSpacePoints,
    interpolateFn: computeFragmentSampleIndex
  });

  t.expectOK(
    checkSampleRectsApproximatelyEqual({
      width,
      height,
      sampleCount,
      actual,
      expected,
      maxDiffULPsForFloatFormat: 1
    })
  );
});

g.test('inputs,front_facing').
desc(
  `
    Test fragment shader builtin(front_facing) values.

    Draws a quad from 2 triangles that entirely cover clip space. (see diagram below in code)
    One triangle is clockwise, the other is counter clockwise. The triangles
    bisect pixels so that different samples are covered by each triangle so that some
    samples should get different values for front_facing for the same fragment.
  `
).
params((u) =>
u //
.combine('nearFar', [[0, 1], [0.25, 0.75]]).
combine('sampleCount', [1, 4]).
combine('frontFace', ['cw', 'ccw']).
combine('interpolation', [
{ type: 'perspective', sampling: 'center' },
{ type: 'perspective', sampling: 'centroid' },
{ type: 'perspective', sampling: 'sample' },
{ type: 'linear', sampling: 'center' },
{ type: 'linear', sampling: 'centroid' },
{ type: 'linear', sampling: 'sample' },
{ type: 'flat', sampling: 'first' },
{ type: 'flat', sampling: 'either' }]
)
).
beforeAllSubcases((t) => {
  const {
    interpolation: { type, sampling }
  } = t.params;
  t.skipIfInterpolationTypeOrSamplingNotSupported({ type, sampling });
}).
fn(async (t) => {
  const {
    nearFar,
    sampleCount,
    frontFace,
    interpolation: { type, sampling }
  } = t.params;
  //
  // We're drawing 2 triangles starting at y = -2 to y = +2
  //
  //  -1   0   1
  //   +===+===+  2
  //   |\  |   |
  //   +---+---+  1  <---
  //   |  \|   |       |
  //   +---+---+  0    | viewport
  //   |   |\  |       |
  //   +---+---+ -1  <---
  //   |   |  \|
  //   +===+===+ -2


  const clipSpacePoints = [
  // ccw
  [-1, -2, 0, 1],
  [1, -2, 0, 1],
  [-1, 2, 0, 1],

  // cw
  [1, -2, 0, 1],
  [-1, 2, 0, 1],
  [1, 2, 0, 1]];


  const interStagePoints = [
  [1, 2, 3, 4],
  [5, 6, 7, 8],
  [9, 10, 11, 12],

  [13, 14, 15, 16],
  [17, 18, 19, 20],
  [21, 22, 23, 24]];


  const width = 4;
  const height = 4;
  const actual = await renderFragmentShaderInputsTo4TexturesAndReadbackValues(t, {
    interpolationType: type,
    interpolationSampling: sampling,
    frontFace,
    sampleCount,
    width,
    height,
    nearFar,
    clipSpacePoints,
    interStagePoints,
    fragInCode: '@builtin(front_facing) frontFacing: bool,',
    outputCode: 'vec4f(select(0.0, 1.0, fin.frontFacing), 0, 0, 0)'
  });

  const expected = generateFragmentInputs({
    width,
    height,
    nearFar,
    sampleCount,
    clipSpacePoints,
    frontFace,
    interpolateFn: computeFragmentFrontFacing
  });

  assert(expected.indexOf(0) >= 0, 'expect some values to be 0');
  assert(expected.findIndex((v) => v !== 0) >= 0, 'expect some values to be non 0');

  t.expectOK(
    checkSampleRectsApproximatelyEqual({
      width,
      height,
      sampleCount,
      actual,
      expected,
      maxDiffULPsForFloatFormat: 0
    })
  );
});

g.test('inputs,sample_mask').
desc(
  `
    Test fragment shader builtin(sample_mask) values.

    Draws various triangles that should trigger different sample_mask values.
    Checks that sample_mask matches what's expected. Note: the triangles
    are selected so they do not intersect sample points as we don't want
    to test precision issues on whether or not a sample point is inside
    or outside the triangle when right on the edge.

    Example: x=-1, y=2, it draws the following triangle

    [ -0.8, -2 ]
    [  1.2,  2 ]
    [ -0.8,  2 ]

    On to a 4x4 pixel texture

     -0.8, 2
      .----------------------.  1.2 2
      |...................../
      |..................../
      |.................../
      |................../
      |................./
    +-|---+-----+-----+/----+  ---
    | |...|.....|...../     |   ^
    | |...|.....|..../|     |   |
    +-|---+-----+---/-+-----+   |
    | |...|.....|../  |     |   |
    | |...|.....|./   |     |   |
    +-|---+-----+/----+-----+   texture / clip space
    | |...|...../     |     |   |
    | |...|..../|     |     |   |
    +-|---+---/-+-----+-----+   |
    | |...|../  |     |     |   |
    | |...|./   |     |     |   V
    +-|---+/----+-----+-----+  ---
      |.../
      |../
      |./
      |/
      /
      .
      -0.8, -2

    Inside an individual pixel you might see this situation

    +-------------+
    |....s1|/     |
    |......|      |
    |...../|   s2 |
    +------C------+
    |s3./  |      |
    |../   |      |
    |./    |s4    |
    +-------------+

    where s1, s2, s3, s4, are sample points and C is the center. For a sampleCount = 4 texture
    the sample_mask is expected to emit sample_mask = 0b0101

    ref: https://learn.microsoft.com/en-us/windows/win32/api/d3d11/ne-d3d11-d3d11_standard_multisample_quality_levels
  `
).
params((u) =>
u //
.combine('nearFar', [[0, 1], [0.25, 0.75]]).
combine('sampleCount', [1, 4]).
combine('interpolation', [
// given that 'sample' effects whether things are run per-sample or per-fragment
// we test all of these to make sure they don't affect the result differently than expected.
{ type: 'perspective', sampling: 'center' },
{ type: 'perspective', sampling: 'centroid' },
{ type: 'perspective', sampling: 'sample' },
{ type: 'linear', sampling: 'center' },
{ type: 'linear', sampling: 'centroid' },
{ type: 'linear', sampling: 'sample' },
{ type: 'flat', sampling: 'first' },
{ type: 'flat', sampling: 'either' }]
).
beginSubcases().
combineWithParams([
{ x: -1, y: -1 },
{ x: -1, y: -2 },
{ x: -1, y: 1 },
{ x: -1, y: 3 },
{ x: -2, y: -1 },
{ x: -2, y: 3 },
{ x: -3, y: -1 },
{ x: -3, y: -2 },
{ x: -3, y: 1 },
{ x: 1, y: -1 },
{ x: 1, y: -3 },
{ x: 1, y: 1 },
{ x: 1, y: 2 },
{ x: 2, y: -2 },
{ x: 2, y: -3 },
{ x: 2, y: 1 },
{ x: 2, y: 2 },
{ x: 3, y: -1 },
{ x: 3, y: -3 },
{ x: 3, y: 1 },
{ x: 3, y: 2 },
{ x: 3, y: 3 }]
)
).
beforeAllSubcases((t) => {
  const {
    interpolation: { type, sampling }
  } = t.params;
  t.skipIfInterpolationTypeOrSamplingNotSupported({ type, sampling });
  t.skipIf(t.isCompatibility, 'sample_mask is not supported in compatibility mode');
}).
fn(async (t) => {
  const {
    x,
    y,
    nearFar,
    sampleCount,
    interpolation: { type, sampling }
  } = t.params;

  const clipSpacePoints = [
  [x + 0.2, -y, 0, 1],
  [-x + 0.2, y, 0, 1],
  [x + 0.2, y, 0, 1]];


  const interStagePoints = [
  [13, 14, 15, 16],
  [17, 18, 19, 20],
  [21, 22, 23, 24]];


  const width = 4;
  const height = 4;
  const actual = await renderFragmentShaderInputsTo4TexturesAndReadbackValues(t, {
    interpolationType: type,
    interpolationSampling: sampling,
    sampleCount,
    width,
    height,
    nearFar,
    clipSpacePoints,
    interStagePoints,
    fragInCode: '@builtin(sample_mask) sample_mask: u32,',
    outputCode: 'vec4f(f32(fin.sample_mask), 0, 0, 0)'
  });

  const expected = generateFragmentInputs({
    width,
    height,
    nearFar,
    sampleCount,
    clipSpacePoints,
    interpolateFn: computeSampleMask
  });

  t.expectOK(
    checkSampleRectsApproximatelyEqual({
      width,
      height,
      sampleCount,
      actual,
      expected,
      maxDiffULPsForFloatFormat: 0
    })
  );
});

const kSizes = [
[15, 15],
[16, 16],
[17, 17],
[19, 13],
[13, 10],
[111, 2],
[2, 111],
[35, 2],
[2, 35],
[53, 13],
[13, 53]];


/**
 * @returns The population count of input.
 *
 * @param input Treated as an unsigned 32-bit integer
 */
function popcount(input) {
  let n = input;
  n = n - (n >> 1 & 0x55555555);
  n = (n & 0x33333333) + (n >> 2 & 0x33333333);
  return (n + (n >> 4) & 0xf0f0f0f) * 0x1010101 >> 24;
}

/**
 * Runs a subgroup builtin test for fragment shaders
 *
 * This test draws a full screen in 2 separate draw calls (half screen each).
 * Results are checked for each draw.
 * @param t The base test
 * @param format The framebuffer format
 * @param fsShader The fragment shader with the following interface:
 *                 Location 0 output is framebuffer with format
 *                 Group 0 binding 0 is a u32 sized data
 * @param width The framebuffer width
 * @param height The framebuffer height
 * @param checker A functor to check the framebuffer values
 */
async function runSubgroupTest(
t,
format,
fsShader,
width,
height,
checker)
{
  const vsShader = `
@vertex
fn vsMain(@builtin(vertex_index) index : u32) -> @builtin(position) vec4f {
  const vertices = array(
    vec2(-1, -1), vec2(-1,  1), vec2( 1,  1),
    vec2(-1, -1), vec2( 1, -1), vec2( 1,  1),
  );
  return vec4f(vec2f(vertices[index]), 0, 1);
}`;

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({ code: vsShader })
    },
    fragment: {
      module: t.device.createShaderModule({ code: fsShader }),
      targets: [{ format }]
    },
    primitive: {
      topology: 'triangle-list'
    }
  });

  const { blockWidth, blockHeight, bytesPerBlock } = getBlockInfoForTextureFormat(format);
  assert(bytesPerBlock !== undefined);

  const blocksPerRow = width / blockWidth;
  const blocksPerColumn = height / blockHeight;
  const bytesPerRow = align(blocksPerRow * (bytesPerBlock ?? 1), 256);
  const byteLength = bytesPerRow * blocksPerColumn;
  const uintLength = byteLength / 4;

  for (let i = 0; i < 2; i++) {
    const framebuffer = t.createTextureTracked({
      size: [width, height],
      usage:
      GPUTextureUsage.COPY_SRC |
      GPUTextureUsage.COPY_DST |
      GPUTextureUsage.RENDER_ATTACHMENT |
      GPUTextureUsage.TEXTURE_BINDING,
      format
    });

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: framebuffer.createView(),
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    pass.setPipeline(pipeline);
    // Draw the upper-left triangle (vertices 0-2) or the lower-right triangle (vertices 3-5)
    pass.draw(3, 1, i * 3);
    pass.end();
    t.queue.submit([encoder.finish()]);

    const buffer = ttu.copyWholeTextureToNewBufferSimple(t, framebuffer, 0);
    const readback = await t.readGPUBufferRangeTyped(buffer, {
      srcByteOffset: 0,
      type: Uint32Array,
      typedLength: uintLength,
      method: 'copy'
    });
    const data = readback.data;

    t.expectOK(checker(data));
  }
}

const kMaximumSubgroupSize = 128;
// A non-zero magic number indicating no expectation error, in order to prevent the false no-error
// result from zero-initialization.
const kSubgroupShaderNoError = 17;

/**
 * Checks subgroup_size builtin value consistency.
 *
 * The builtin subgroup_size is not assumed to be uniform in fragment shaders.
 * Therefore, this function checks the value is a power of two within the device
 * limits and that the ballot size is less than the stated size.
 * @param data An array of vec4u that contains (per texel):
 *             * subgroup_size builtin value
 *             * balloted active invocations number
 *             * balloted subgroup size all active invocations agreed on, otherwise 0
 *             * error flag, should be equal to kSubgroupShaderNoError or shader found
 *               expectation failed otherwise.
 * @param format The texture format for data
 * @param min The minimum subgroup size from the device
 * @param max The maximum subgroup size from the device
 * @param width The width of the framebuffer
 * @param height The height of the framebuffer
 */
function checkSubgroupSizeConsistency(
data,
format,
min,
max,
width,
height)
{
  const { blockWidth, blockHeight, bytesPerBlock } = getBlockInfoForTextureFormat(format);
  const blocksPerRow = width / blockWidth;
  // Image copies require bytesPerRow to be a multiple of 256.
  const bytesPerRow = align(blocksPerRow * (bytesPerBlock ?? 1), 256);
  const uintsPerRow = bytesPerRow / 4;
  const uintsPerTexel = (bytesPerBlock ?? 1) / blockWidth / blockHeight / 4;

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const offset = uintsPerRow * row + col * uintsPerTexel;
      const subgroupSize = data[offset];
      const countActive = data[offset + 1];
      const ballotedSubgroupSize = data[offset + 2];
      const error = data[offset + 3];

      if (error === 0) {
        // Inactive fragment get error `0` instead of noError. Check all output being zero.
        if (subgroupSize !== 0 || countActive !== 0 || ballotedSubgroupSize !== 0) {
          return new Error(
            `Unexpected zero error with non-zero outputs for (${row}, ${col}): got output [${subgroupSize}, ${countActive}, ${ballotedSubgroupSize}, ${error}]`
          );
        }
        continue;
      }

      if (popcount(subgroupSize) !== 1) {
        return new Error(`Subgroup size '${subgroupSize}' is not a power of two`);
      }

      if (subgroupSize < min) {
        return new Error(`Subgroup size '${subgroupSize}' is less than minimum '${min}'`);
      }
      if (max < subgroupSize) {
        return new Error(`Subgroup size '${subgroupSize}' is greater than maximum '${max}'`);
      }

      if (subgroupSize < countActive) {
        return new Error(`Unexpected active invocations number larger than subgroup size
-       icoord: (${row}, ${col})
- subgroupSize: ${subgroupSize}
-  countActive: ${countActive}`);
      }

      if (subgroupSize !== ballotedSubgroupSize) {
        return new Error(`Inconsistent subgroup size
-                 icoord: (${row}, ${col})
-           subgroupSize: ${subgroupSize}
- balloted subgroup size: ${ballotedSubgroupSize}`);
      }

      if (error !== kSubgroupShaderNoError) {
        return new Error(
          `Unexpected error value
-   icoord: (${row}, ${col})
- expected: noError (${kSubgroupShaderNoError})
-      got: ${error}`
        );
      }
    }
  }

  return undefined;
}

g.test('subgroup_size').
desc('Tests subgroup_size values').
params((u) =>
u.
combine('size', kSizes).
beginSubcases().
combineWithParams([{ format: 'rgba32uint' }])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');




  const { subgroupMinSize, subgroupMaxSize } = t.device.adapterInfo;

  const fsShader = `
enable subgroups;

const subgroupMaxSize = ${kMaximumSubgroupSize}u;
const noError = ${kSubgroupShaderNoError}u;

const width = ${t.params.size[0]};
const height = ${t.params.size[1]};

@fragment
fn fsMain(
  @builtin(position) pos : vec4f,
  @builtin(subgroup_size) sg_size : u32,
) -> @location(0) vec4u {
  var error: u32 = noError;

  let ballotActive = countOneBits(subgroupBallot(true));
  let countActive = ballotActive.x + ballotActive.y + ballotActive.z + ballotActive.w;
  // Validate that balloted active invocations number no larger than subgroup size
  if (countActive > sg_size) {
    error++;
  }

  var subgroupSizeBallotedInvocations: u32 = 0u;
  var ballotedSubgroupSize: u32 = 0u;
  for (var i: u32 = 0; i <= subgroupMaxSize; i++) {
    let ballotSubgroupSizeEqualI = countOneBits(subgroupBallot(sg_size == i));
    let countSubgroupSizeEqualI = ballotSubgroupSizeEqualI.x + ballotSubgroupSizeEqualI.y + ballotSubgroupSizeEqualI.z + ballotSubgroupSizeEqualI.w;
    subgroupSizeBallotedInvocations += countSubgroupSizeEqualI;
    // Validate that all active invocations see the same subgroup size, i.e. ballotedSubgroupSize
    ballotedSubgroupSize = select(ballotedSubgroupSize, i, countSubgroupSizeEqualI == countActive);
    error = select(error, error + 1, countSubgroupSizeEqualI != countActive && countSubgroupSizeEqualI != 0);
  }
  // Validate that all active invocations balloted in previous loop
  if (subgroupSizeBallotedInvocations != countActive) {
    error++;
  }
  // Validate that ballotedSubgroupSize is identical to subgroup_size
  if (ballotedSubgroupSize != sg_size) {
    error++;
  }

  return vec4u(sg_size, countActive, ballotedSubgroupSize, error);
}`;

  await runSubgroupTest(
    t,
    t.params.format,
    fsShader,
    t.params.size[0],
    t.params.size[1],
    (data) => {
      return checkSubgroupSizeConsistency(
        data,
        t.params.format,
        subgroupMinSize,
        subgroupMaxSize,
        t.params.size[0],
        t.params.size[1]
      );
    }
  );
});

/**
 * Checks subgroup_invocation_id value consistency
 *
 * Very little uniformity is expected for subgroup_invocation_id.
 * This function checks that all ids are less than the subgroup size
 * (not the ballot size, since the subgroup id can be allocated to
 * inactivate invocations between active ones) and no id is repeated.
 * @param data An array of vec4u that contains (per texel):
 *             * subgroup_invocation_id
 *             * subgroup size
 *             * ballot active invocation number
 *             * error flag, should be equal to kSubgroupShaderNoError or shader found
 *               expectation failed otherwise.
 * @param format The texture format of data
 * @param width The width of the framebuffer
 * @param height The height of the framebuffer
 */
function checkSubgroupInvocationIdConsistency(
data,
format,
width,
height)
{
  const { blockWidth, blockHeight, bytesPerBlock } = getBlockInfoForTextureFormat(format);
  const blocksPerRow = width / blockWidth;
  const bytesPerRow = align(blocksPerRow * (bytesPerBlock ?? 1), 256);
  const uintsPerRow = bytesPerRow / 4;
  const uintsPerTexel = (bytesPerBlock ?? 1) / blockWidth / blockHeight / 4;

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const offset = uintsPerRow * row + col * uintsPerTexel;
      const id = data[offset];
      const sgSize = data[offset + 1];
      const ballotSize = data[offset + 2];
      const error = data[offset + 3];

      if (error === 0) {
        // Inactive fragment get error `0` instead of noError. Check all output being zero.
        if (id !== 0 || sgSize !== 0 || ballotSize !== 0) {
          return new Error(
            `Unexpected zero error with non-zero outputs for (${row}, ${col}): got output [${id}, ${sgSize}, ${ballotSize}, ${error}]`
          );
        }
        continue;
      }

      if (sgSize < id) {
        return new Error(
          `Invocation id '${id}' is greater than subgroup size '${sgSize}' for (${row}, ${col})`
        );
      }

      if (sgSize < ballotSize) {
        return new Error(
          `Ballot size '${ballotSize}' is greater than subgroup size '${sgSize}' for (${row}, ${col})`
        );
      }

      if (error !== kSubgroupShaderNoError) {
        return new Error(
          `Unexpected error value
-   icoord: (${row}, ${col})
- expected: noError (${kSubgroupShaderNoError})
-      got: ${error}`
        );
      }
    }
  }

  return undefined;
}

g.test('subgroup_invocation_id').
desc('Tests subgroup_invocation_id built-in value').
params((u) =>
u.
combine('size', kSizes).
beginSubcases().
combineWithParams([{ format: 'rgba32uint' }])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const fsShader = `
enable subgroups;

const width = ${t.params.size[0]};
const height = ${t.params.size[1]};

const subgroupMaxSize = ${kMaximumSubgroupSize}u;
// A non-zero magic number indicating no expectation error, in order to prevent the
// false no-error result from zero-initialization.
const noError = ${kSubgroupShaderNoError}u;

@fragment
fn fsMain(
  @builtin(position) pos : vec4f,
  @builtin(subgroup_invocation_id) id : u32,
  @builtin(subgroup_size) sg_size : u32,
) -> @location(0) vec4u {

  var error: u32 = noError;

  // Validate that reported subgroup size is no larger than subgroupMaxSize
  if (sg_size > subgroupMaxSize) {
    error++;
  }

  // Validate that reported subgroup invocation id is smaller than subgroup size
  if (id >= sg_size) {
    error++;
  }

  // Validate that each subgroup id is assigned to at most one active invocation
  // in the subgroup
  var countAssignedId: u32 = 0u;
  for (var i: u32 = 0; i < subgroupMaxSize; i++) {
    let ballotIdEqualsI = countOneBits(subgroupBallot(id == i));
    let countInvocationIdEqualsI = ballotIdEqualsI.x + ballotIdEqualsI.y + ballotIdEqualsI.z + ballotIdEqualsI.w;
    // Validate an id assigned at most once
    error += select(1u, 0u, countInvocationIdEqualsI <= 1);
    // Validate id larger than subgroup size will not get balloted
    error += select(1u, 0u, (id < sg_size) || (countInvocationIdEqualsI == 0));
    // Sum up the assigned invocation number of each id
    countAssignedId += countInvocationIdEqualsI;
  }
  // Validate that all active invocation get counted during the above loop
  let ballotActive = countOneBits(subgroupBallot(true));
  let activeInvocations = ballotActive.x + ballotActive.y + ballotActive.z + ballotActive.w;
  if (activeInvocations != countAssignedId) {
    error++;
  }

  return vec4u(id, sg_size, activeInvocations, error);
}`;

  await runSubgroupTest(
    t,
    t.params.format,
    fsShader,
    t.params.size[0],
    t.params.size[1],
    (data) => {
      return checkSubgroupInvocationIdConsistency(
        data,
        t.params.format,
        t.params.size[0],
        t.params.size[1]
      );
    }
  );
});

/**
 * Checks primitive_index value consistency
 *
 * Renders fullscreen triangles using the given draw arguments, writing the
 * primitive_index of each to the render target. Then reads back the texture and
 * compares the last primitive_index written to the expected value. All args are
 * passed directly to draw/drawIndexed unless specified otherwise.
 * @param indices An array of indices to be used as a 32 bit index buffer.
 *                Causes drawIndexed to be used instead of draw.
 * @param topology The primitive topology to use.
 * @param expected The expected value of the last primitive_index drawn.
 */
function runPrimitiveIndexTest(
t,
{
  count,
  instances = 1,
  firstVertex = 0,
  firstInstance = 0,
  firstIndex = 0,
  vertices = null,
  indices = null,
  topology = 'triangle-list',
  cullMode = 'none',
  width = 4,
  height = 4,
  expected













})
{
  const shader = `
enable primitive_index;

@vertex
fn vsFullscreenMain(@builtin(vertex_index) index : u32) -> @builtin(position) vec4f {
  const vertices = array(
    vec2(-1, -1), vec2( 3,  -1), vec2(-1,  3),
  );
  return vec4f(vec2f(vertices[index%3]), 0, 1);
}

@vertex
fn vsBufferMain(@builtin(vertex_index) index : u32, @location(0) pos : vec2f) -> @builtin(position) vec4f {
  return vec4f(pos, 0, 1);
}

@fragment
fn fsMain(@builtin(primitive_index) pid : u32) -> @location(0) vec4u {
  return vec4u(pid, 0, 0, 0);
}`;

  const format = 'r32uint';

  const module = t.device.createShaderModule({ code: shader });

  const buffers = [];

  if (vertices) {
    buffers.push({
      arrayStride: Float32Array.BYTES_PER_ELEMENT * 2,
      attributes: [
      {
        format: 'float32x2',
        offset: 0,
        shaderLocation: 0
      }]

    });
  }

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module,
      entryPoint: vertices ? 'vsBufferMain' : 'vsFullscreenMain',
      buffers
    },
    fragment: {
      module,
      targets: [{ format }]
    },
    primitive: {
      topology,
      cullMode,
      stripIndexFormat: topology.includes('list') ? undefined : 'uint32'
    }
  });

  const framebuffer = t.createTextureTracked({
    size: [width, height],
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.COPY_DST |
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.TEXTURE_BINDING,
    format
  });

  let vertexBuffer = null;
  if (vertices) {
    vertexBuffer = t.createBufferTracked({
      size: vertices.length * Float32Array.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.VERTEX,
      mappedAtCreation: true
    });
    const float32Array = new Float32Array(vertexBuffer.getMappedRange());
    float32Array.set(vertices);
    vertexBuffer.unmap();
  }

  let indexBuffer = null;
  if (indices) {
    indexBuffer = t.createBufferTracked({
      size: indices.length * Uint32Array.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.INDEX,
      mappedAtCreation: true
    });
    const uint32Array = new Uint32Array(indexBuffer.getMappedRange());
    uint32Array.set(indices);
    indexBuffer.unmap();
  }

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: framebuffer.createView(),
      loadOp: 'clear',
      clearValue: [0xffffffff, 0, 0, 0], // Clear to max uint32 to ensure that primitive id 0 is testable
      storeOp: 'store'
    }]

  });
  pass.setPipeline(pipeline);
  // Draw the primitives
  if (vertexBuffer) {
    pass.setVertexBuffer(0, vertexBuffer);
  }

  if (indexBuffer) {
    pass.setIndexBuffer(indexBuffer, 'uint32');
    pass.drawIndexed(count, instances, firstIndex, firstVertex, firstInstance);
  } else {
    pass.draw(count, instances, firstVertex, firstInstance);
  }
  pass.end();
  t.queue.submit([encoder.finish()]);

  if (Array.isArray(expected)) {
    ttu.expectSinglePixelComparisonsAreOkInTexture(t, { texture: framebuffer }, expected);
  } else {
    ttu.expectSingleColorWithTolerance(t, framebuffer, format, {
      size: [width, height, 1],
      exp: { R: expected },
      layout: { mipLevel: 0 },
      maxFractionalDiff: 0
    });
  }
}

g.test('primitive_index,basic').
desc('Tests primitive_index built-in value').
params((u) =>
u.
beginSubcases().
combine('triCount', [1, 4, 16])
// None of the following should affect the primitive_index
.combine('instances', [1, 4, 16]).
combine('firstVertex', [0, 1, 4]).
combine('firstIndex', [0, 3, 9]).
combine('firstInstance', [0, 1, 4])
).
fn((t) => {
  const { triCount, instances, firstVertex, firstIndex, firstInstance } = t.params;
  t.skipIfDeviceDoesNotHaveFeature('primitive-index');

  runPrimitiveIndexTest(t, {
    count: triCount * 3,
    instances,
    firstVertex,
    firstInstance,
    expected: triCount - 1
  });

  const indices = [];
  for (let i = 0; i < triCount + Math.ceil(firstIndex / 3); ++i) {
    indices.push(0, 1, 2);
  }

  runPrimitiveIndexTest(t, {
    count: triCount * 3,
    instances,
    firstVertex,
    firstInstance,
    firstIndex,
    indices,
    expected: triCount - 1
  });
});

g.test('primitive_index,primitive_reset').
desc(
  'Tests that the primitive_index built-in value does not increment or reset across primitive resets'
).
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('primitive-index');

  runPrimitiveIndexTest(t, {
    count: 10,
    topology: 'triangle-strip',
    indices: [0, 1, 2, 0, 1, 0xffffffff, 0, 1, 2, 0],
    expected: 4
  });
});

g.test('primitive_index,discarded_primitves').
desc(
  'Tests that the primitives which are discarded due to culling, size, or shape still increment the primitive_index built-in'
).
params((u) =>
u.beginSubcases().combine('vertices', [
[0.3, 0.3, 0.3, 0.3, 0.3, 0.3], // Zero size triangle
[0.3, 0.3, 0.3, 0.3, 0.3, 1.3], // Degenerate triangle
[0.3, 0.3, 0.30001, 0.3, 0.3, 0.30001], // Sub-pixel triangle
[2, 2, 2, 3, 3, 2], // Offscreen triangle
[-1, -1, -1, 3, 3, -1] // Backface culled triangle
])
).
fn((t) => {
  const { vertices } = t.params;
  t.skipIfDeviceDoesNotHaveFeature('primitive-index');

  runPrimitiveIndexTest(t, {
    count: 6,
    vertices: [...vertices, -1, -1, 3, -1, -1, 3], // Append a fulscreen triangle to the test vertices
    cullMode: 'back',
    expected: 1
  });
});

g.test('primitive_index,topologies').
desc('Tests that the primitive_index built-in value works every topology').
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('primitive-index');

  const triListVertices = [
  //           0,2
  //            +
  //           /|
  //          /.|
  //         +--+--+
  //        /|..|  |
  //       /.|..|  |
  // -2,0 +--+--O--+--+ 2,0
  //         |  |  |
  //         |  |  |
  //         +--+--+
  //            |
  //            |
  //            +
  //           0,-2
  0, 0, -2, 0, 0, 2,

  //           0,2
  //            +
  //            |\
  //            |.\
  //         +--+--+
  //         |  |..|\
  //         |  |..|.\
  // -2,0 +--+--O--+--+ 2,0
  //         |  |  |
  //         |  |  |
  //         +--+--+
  //            |
  //            |
  //            +
  //           0,-2
  0, 0, 0, 2, 2, 0,

  //           0,2
  //            +
  //            |
  //            |
  //         +--+--+
  //         |  |  |
  //         |  |  |
  // -2,0 +--+--O--+--+ 2,0
  //       \.|..|  |
  //        \|..|  |
  //         +--+--+
  //          \.|
  //           \|
  //            +
  //           0,-2
  0, 0, -2, 0, 0, -2,

  //           0,2
  //            +
  //            |
  //            |
  //         +--+--+
  //         |  |  |
  //         |  |  |
  // -2,0 +--+--O--+--+ 2,0
  //         |  |..|./
  //         |  |..|/
  //         +--+--+
  //            |./
  //            |/
  //            +
  //           0,-2
  0, 0, 0, -2, 2, 0];

  runPrimitiveIndexTest(t, {
    count: 12,
    topology: 'triangle-list',
    vertices: triListVertices,
    width: 2,
    height: 2,
    expected: [
    { coord: [0, 0, 0], exp: { R: 0 } },
    { coord: [1, 0, 0], exp: { R: 1 } },
    { coord: [0, 1, 0], exp: { R: 2 } },
    { coord: [1, 1, 0], exp: { R: 3 } }]

  });

  //         v2
  //          +
  //          |
  //          |
  //       +--+--+
  //       |  |  |
  //       |v1|v4|
  // v0 +--+--.--+--+ v3
  // v7    |  |  |
  //       |  |  |
  //       +--+--+
  //          |
  //          |
  //          +
  //          v5,v6
  //
  //  #0  #1  #2  #3  #4  #5
  //   +  +   +   +-+ +   +-+
  //  /|  |\  |\  |/  |    \|
  // +-+  +-+ +-+ +   +     +
  const triStripVertices = [-2, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, -2, 0, 0, -2, 0];
  runPrimitiveIndexTest(t, {
    count: 8,
    topology: 'triangle-strip',
    vertices: triStripVertices,
    width: 2,
    height: 2,
    expected: [
    { coord: [0, 0, 0], exp: { R: 0 } },
    { coord: [1, 0, 0], exp: { R: 2 } },
    { coord: [1, 1, 0], exp: { R: 3 } },
    { coord: [0, 1, 0], exp: { R: 5 } }]

  });

  //   v1,v5   v2,v6
  // +---*---+---*---+
  // |       |       |
  // |       |       |
  // |       |       |
  // +-------o-------+
  // |       |       |
  // |       |       |
  // |       |       |
  // +---*---+---*---+
  //   v0,v4   v3,v7
  const lineVertices = [-0.5, -1, -0.5, 1, 0.5, 1, 0.5, -1, -0.5, -1, -0.5, 1, 0.5, 1, 0.5, -1];
  runPrimitiveIndexTest(t, {
    count: 4,
    topology: 'line-list',
    vertices: lineVertices,
    width: 2,
    expected: [
    { coord: [0, 0, 0], exp: { R: 0 } },
    { coord: [0, 1, 0], exp: { R: 0 } },
    { coord: [1, 0, 0], exp: { R: 1 } },
    { coord: [1, 1, 0], exp: { R: 1 } }]

  });

  runPrimitiveIndexTest(t, {
    count: 8,
    topology: 'line-list',
    vertices: lineVertices,
    width: 2,
    expected: [
    { coord: [0, 0, 0], exp: { R: 2 } },
    { coord: [0, 1, 0], exp: { R: 2 } },
    { coord: [1, 0, 0], exp: { R: 3 } },
    { coord: [1, 1, 0], exp: { R: 3 } }]

  });

  runPrimitiveIndexTest(t, {
    count: 4,
    topology: 'line-strip',
    vertices: lineVertices,
    width: 2,
    expected: [
    { coord: [0, 0, 0], exp: { R: 0 } },
    { coord: [0, 1, 0], exp: { R: 0 } },
    { coord: [1, 0, 0], exp: { R: 2 } },
    { coord: [1, 1, 0], exp: { R: 2 } }]

  });

  runPrimitiveIndexTest(t, {
    count: 8,
    topology: 'line-strip',
    vertices: lineVertices,
    width: 2,
    expected: [
    { coord: [0, 0, 0], exp: { R: 4 } },
    { coord: [0, 1, 0], exp: { R: 4 } },
    { coord: [1, 0, 0], exp: { R: 6 } },
    { coord: [1, 1, 0], exp: { R: 6 } }]

  });

  //   v1,v5   v2,v6
  // +-------+-------+
  // |       |       |
  // |   *   |   *   |
  // |       |       |
  // +-------o-------+
  // |       |       |
  // |   *   |   *   |
  // |       |       |
  // +-------+-------+
  //   v0,v4   v3,v7
  const pointVertices = [
  -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, 0.5];

  runPrimitiveIndexTest(t, {
    count: 4,
    topology: 'point-list',
    vertices: pointVertices,
    width: 2,
    height: 2,
    expected: [
    { coord: [0, 1, 0], exp: { R: 0 } },
    { coord: [1, 1, 0], exp: { R: 1 } },
    { coord: [0, 0, 0], exp: { R: 2 } },
    { coord: [1, 0, 0], exp: { R: 3 } }]

  });

  runPrimitiveIndexTest(t, {
    count: 8,
    topology: 'point-list',
    vertices: pointVertices,
    width: 2,
    height: 2,
    expected: [
    { coord: [0, 1, 0], exp: { R: 4 } },
    { coord: [1, 1, 0], exp: { R: 5 } },
    { coord: [0, 0, 0], exp: { R: 6 } },
    { coord: [1, 0, 0], exp: { R: 7 } }]

  });
});