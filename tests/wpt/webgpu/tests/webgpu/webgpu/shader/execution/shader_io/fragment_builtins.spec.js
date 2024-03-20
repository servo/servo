/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Test fragment shader builtin variables and inter-stage variables

* test builtin(position)
* test @interpolate
* test builtin(sample_index)
* test builtin(front_facing)

Note: @interpolate settings and sample_index affect whether or not the fragment shader
is evaluated per-fragment or per-sample. With @interpolate(, sample) or usage of
@builtin(sample_index) the fragment shader should be executed per-sample.

TODO:
* test frag_depth
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ErrorWithExtra, assert, range, unreachable } from '../../../../common/util/util.js';

import { GPUTest } from '../../../gpu_test.js';
import { getMultisampleFragmentOffsets } from '../../../multisample_info.js';
import { dotProduct, subtractVectors } from '../../../util/math.js';
import { TexelView } from '../../../util/texture/texel_view.js';
import { findFailedPixels } from '../../../util/texture/texture_ok.js';

export const g = makeTestGroup(GPUTest);

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

  const copyBuffer = t.device.createBuffer({
    size: width * height * sampleCount * 4 * 4,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });
  t.trackForCleanup(copyBuffer);

  const buffer = t.device.createBuffer({
    size: copyBuffer.size,
    usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST
  });
  t.trackForCleanup(buffer);

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
function createInterStageInterpolationFn(
interStagePoints,
type,
sampling)
{
  return function ({
    baseVertexIndex,
    fragmentBarycentricCoords,
    sampleBarycentricCoords,
    clipSpacePoints
  }) {
    const triangleInterStagePoints = interStagePoints.slice(baseVertexIndex, baseVertexIndex + 3);
    const barycentricCoords =
    sampling === 'center' ? fragmentBarycentricCoords : sampleBarycentricCoords;
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
        return triangleInterStagePoints[0];
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
function createInterStageInterpolationBetween0And1TestFn(
interStagePoints,
type,
sampling)
{
  const interpolateFn = createInterStageInterpolationFn(interStagePoints, type, sampling);
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
 * We can't use rgba32float because it's optional. We can't use rgba16float
 * because it's optional in compat. We can't we use rgba32uint as that can't be
 * multisampled.
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

  const textures = range(4, () => {
    const texture = t.device.createTexture({
      size: [width, height],
      usage:
      GPUTextureUsage.RENDER_ATTACHMENT |
      GPUTextureUsage.TEXTURE_BINDING |
      GPUTextureUsage.COPY_SRC,
      format: 'rgba8unorm',
      sampleCount
    });
    t.trackForCleanup(texture);
    return texture;
  });

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

  const uniformBuffer = t.device.createBuffer({
    size: 8,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
  });
  t.trackForCleanup(uniformBuffer);
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
{ type: 'flat' }]
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
{ type: 'perspective', sampling: 'center' },
{ type: 'perspective', sampling: 'sample' },
{ type: 'linear', sampling: 'center' },
{ type: 'linear', sampling: 'sample' },
{ type: 'flat' }]
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
    interpolateFn: createInterStageInterpolationFn(interStagePoints, type, sampling)
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
    interpolateFn: createInterStageInterpolationBetween0And1TestFn(
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
{ type: 'flat' }]
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
{ type: 'flat' }]
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

  // Double check, first corner should be different than last based on the triangles we are drawing.
  assert(expected[0] !== expected[expected.length - 4]);

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