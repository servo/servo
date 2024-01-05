/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
- Test pipeline outputs with different color attachment number, formats, component counts, etc.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { range } from '../../../../common/util/util.js';
import {
  computeBytesPerSampleFromFormats,
  kRenderableColorTextureFormats,
  kTextureFormatInfo } from
'../../../format_info.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';
import { getFragmentShaderCodeWithOutput, getPlainTypeInfo } from '../../../util/shader.js';
import { kTexelRepresentationInfo } from '../../../util/texture/texel_data.js';

const kVertexShader = `
@vertex fn main(
@builtin(vertex_index) VertexIndex : u32
) -> @builtin(position) vec4<f32> {
  var pos : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
      vec2<f32>(-1.0, -3.0),
      vec2<f32>(3.0, 1.0),
      vec2<f32>(-1.0, 1.0));
  return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
}
`;

export const g = makeTestGroup(TextureTestMixin(GPUTest));

// Values to write into each attachment
// We make values different for each attachment index and each channel
// to make sure they didn't get mixed up

// Clamp alpha to 3 to avoid comparing a large expected value with a max 3 value for rgb10a2uint
// MAINTENANCE_TODO: Make TexelRepresentation.numericRange per-component and use that.
const attachmentsIntWriteValues = [
{ R: 1, G: 2, B: 3, A: 1 },
{ R: 5, G: 6, B: 7, A: 2 },
{ R: 9, G: 10, B: 11, A: 3 },
{ R: 13, G: 14, B: 15, A: 0 }];

const attachmentsFloatWriteValues = [
{ R: 0.12, G: 0.34, B: 0.56, A: 0 },
{ R: 0.78, G: 0.9, B: 0.19, A: 1 },
{ R: 0.28, G: 0.37, B: 0.46, A: 0.3 },
{ R: 0.55, G: 0.64, B: 0.73, A: 1 }];


g.test('color,attachments').
desc(`Test that pipeline with sparse color attachments write values correctly.`).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).
beginSubcases().
combine('attachmentCount', [2, 3, 4]).
expand('emptyAttachmentId', (p) => range(p.attachmentCount, (i) => i))
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { format, attachmentCount, emptyAttachmentId } = t.params;
  const componentCount = kTexelRepresentationInfo[format].componentOrder.length;
  const info = kTextureFormatInfo[format];

  // We only need to test formats that have a valid color attachment bytes per sample.
  const pixelByteCost = kTextureFormatInfo[format].colorRender?.byteCost;
  t.skipIf(
    pixelByteCost === undefined ||
    computeBytesPerSampleFromFormats(range(attachmentCount, () => format)) >
    t.device.limits.maxColorAttachmentBytesPerSample
  );

  const writeValues =
  info.color.type === 'sint' || info.color.type === 'uint' ?
  attachmentsIntWriteValues :
  attachmentsFloatWriteValues;

  const renderTargets = range(attachmentCount, () =>
  t.device.createTexture({
    format,
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  })
  );
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: kVertexShader
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: getFragmentShaderCodeWithOutput(
          range(attachmentCount, (i) =>
          i === emptyAttachmentId ?
          null :
          {
            values: [
            writeValues[i].R,
            writeValues[i].G,
            writeValues[i].B,
            writeValues[i].A],

            plainType: getPlainTypeInfo(info.color.type),
            componentCount
          }
          )
        )
      }),
      entryPoint: 'main',
      targets: range(attachmentCount, (i) => i === emptyAttachmentId ? null : { format })
    },
    primitive: { topology: 'triangle-list' }
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: range(attachmentCount, (i) =>
    i === emptyAttachmentId ?
    null :
    {
      view: renderTargets[i].createView(),
      storeOp: 'store',
      clearValue: { r: 0.5, g: 0.5, b: 0.5, a: 0.5 },
      loadOp: 'clear'
    }
    )
  });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  for (let i = 0; i < attachmentCount; i++) {
    if (i === emptyAttachmentId) {
      continue;
    }
    t.expectSinglePixelComparisonsAreOkInTexture({ texture: renderTargets[i] }, [
    { coord: { x: 0, y: 0 }, exp: writeValues[i] }]
    );
  }
});

g.test('color,component_count').
desc(
  `Test that extra components of the output (e.g. f32, vec2<f32>, vec3<f32>, vec4<f32>) are discarded.`
).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).
beginSubcases().
combine('componentCount', [1, 2, 3, 4]).
filter((x) => x.componentCount >= kTexelRepresentationInfo[x.format].componentOrder.length)
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { format, componentCount } = t.params;
  const info = kTextureFormatInfo[format];

  // expected RGBA values
  // extra channels are discarded
  const values = [0, 1, 0, 1];

  const renderTarget = t.device.createTexture({
    format,
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: kVertexShader
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: getFragmentShaderCodeWithOutput([
        {
          values,
          plainType: getPlainTypeInfo(info.color.type),
          componentCount
        }]
        )
      }),
      entryPoint: 'main',
      targets: [{ format }]
    },
    primitive: { topology: 'triangle-list' }
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: renderTarget.createView(),
      storeOp: 'store',
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear'
    }]

  });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  t.expectSingleColor(renderTarget, format, {
    size: [1, 1, 1],
    exp: { R: values[0], G: values[1], B: values[2], A: values[3] }
  });
});

g.test('color,component_count,blend').
desc(
  `Test that blending behaves correctly when:
- fragment output has no alpha, but the src alpha is not used for the blend operation indicated by blend factors
- attachment format has no alpha, and the dst alpha should be assumed as 1

The attachment has a load value of [1, 0, 0, 1]
`
).
params((u) =>
u.
combine('format', ['r8unorm', 'rg8unorm', 'rgba8unorm', 'bgra8unorm']).
beginSubcases()
// _result is expected values in the color attachment (extra channels are discarded)
// output is the fragment shader output vector
// 0.498 -> 0x7f, 0.502 -> 0x80
.combineWithParams([
// fragment output has no alpha
{
  _result: [0, 0, 0, 0],
  output: [0],
  colorSrcFactor: 'one',
  colorDstFactor: 'zero',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'zero'
},
{
  _result: [0, 0, 0, 0],
  output: [0],
  colorSrcFactor: 'dst-alpha',
  colorDstFactor: 'zero',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'zero'
},
{
  _result: [1, 0, 0, 0],
  output: [0],
  colorSrcFactor: 'one-minus-dst-alpha',
  colorDstFactor: 'dst-alpha',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'one'
},
{
  _result: [0.498, 0, 0, 0],
  output: [0.498],
  colorSrcFactor: 'dst-alpha',
  colorDstFactor: 'zero',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'one'
},
{
  _result: [0, 1, 0, 0],
  output: [0, 1],
  colorSrcFactor: 'one',
  colorDstFactor: 'zero',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'zero'
},
{
  _result: [0, 1, 0, 0],
  output: [0, 1],
  colorSrcFactor: 'dst-alpha',
  colorDstFactor: 'zero',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'zero'
},
{
  _result: [1, 0, 0, 0],
  output: [0, 1],
  colorSrcFactor: 'one-minus-dst-alpha',
  colorDstFactor: 'dst-alpha',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'one'
},
{
  _result: [0, 1, 0, 0],
  output: [0, 1, 0],
  colorSrcFactor: 'one',
  colorDstFactor: 'zero',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'zero'
},
{
  _result: [0, 1, 0, 0],
  output: [0, 1, 0],
  colorSrcFactor: 'dst-alpha',
  colorDstFactor: 'zero',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'zero'
},
{
  _result: [1, 0, 0, 0],
  output: [0, 1, 0],
  colorSrcFactor: 'one-minus-dst-alpha',
  colorDstFactor: 'dst-alpha',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'one'
},
// fragment output has alpha
{
  _result: [0.502, 1, 0, 0.498],
  output: [0, 1, 0, 0.498],
  colorSrcFactor: 'one',
  colorDstFactor: 'one-minus-src-alpha',
  alphaSrcFactor: 'one',
  alphaDstFactor: 'zero'
},
{
  _result: [0.502, 0.498, 0, 0.498],
  output: [0, 1, 0, 0.498],
  colorSrcFactor: 'src-alpha',
  colorDstFactor: 'one-minus-src-alpha',
  alphaSrcFactor: 'one',
  alphaDstFactor: 'zero'
},
{
  _result: [0, 1, 0, 0.498],
  output: [0, 1, 0, 0.498],
  colorSrcFactor: 'dst-alpha',
  colorDstFactor: 'zero',
  alphaSrcFactor: 'one',
  alphaDstFactor: 'zero'
},
{
  _result: [0, 1, 0, 0.498],
  output: [0, 1, 0, 0.498],
  colorSrcFactor: 'dst-alpha',
  colorDstFactor: 'zero',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'src'
},
{
  _result: [1, 0, 0, 1],
  output: [0, 1, 0, 0.498],
  colorSrcFactor: 'one-minus-dst-alpha',
  colorDstFactor: 'dst-alpha',
  alphaSrcFactor: 'zero',
  alphaDstFactor: 'dst-alpha'
}]
).
filter((x) => x.output.length >= kTexelRepresentationInfo[x.format].componentOrder.length)
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const {
    format,
    _result,
    output,
    colorSrcFactor,
    colorDstFactor,
    alphaSrcFactor,
    alphaDstFactor
  } = t.params;
  const componentCount = output.length;
  const info = kTextureFormatInfo[format];

  const renderTarget = t.device.createTexture({
    format,
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: kVertexShader
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: getFragmentShaderCodeWithOutput([
        {
          values: output,
          plainType: getPlainTypeInfo(info.color.type),
          componentCount
        }]
        )
      }),
      entryPoint: 'main',
      targets: [
      {
        format,
        blend: {
          color: {
            srcFactor: colorSrcFactor,
            dstFactor: colorDstFactor,
            operation: 'add'
          },
          alpha: {
            srcFactor: alphaSrcFactor,
            dstFactor: alphaDstFactor,
            operation: 'add'
          }
        }
      }]

    },
    primitive: { topology: 'triangle-list' }
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: renderTarget.createView(),
      storeOp: 'store',
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear'
    }]

  });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  t.expectSingleColor(renderTarget, format, {
    size: [1, 1, 1],
    exp: { R: _result[0], G: _result[1], B: _result[2], A: _result[3] }
  });
});