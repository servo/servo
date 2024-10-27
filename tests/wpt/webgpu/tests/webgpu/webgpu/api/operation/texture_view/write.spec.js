/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test the result of writing textures through texture views with various options.

Reads value from a shader array, writes the value via various write methods.
Check the texture result with the expected texel view.

All x= every possible view write method: {
  - storage write {fragment, compute}
  - render pass store
  - render pass resolve
}

Format reinterpretation is not tested here. It is in format_reinterpretation.spec.ts.

TODO: Write helper for this if not already available (see resource_init, buffer_sync_test for related code).
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { unreachable } from '../../../../common/util/util.js';
import {
  kRegularTextureFormats,
  kTextureFormatInfo } from

'../../../format_info.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';
import { kFullscreenQuadVertexShaderCode } from '../../../util/shader.js';
import { TexelView } from '../../../util/texture/texel_view.js';

export const g = makeTestGroup(TextureTestMixin(GPUTest));

const kTextureViewWriteMethods = [
'storage-write-fragment',
'storage-write-compute',
'render-pass-store',
'render-pass-resolve'];



const kTextureViewUsageMethods = ['inherit', 'minimal'];


// Src color values to read from a shader array.
const kColorsFloat = [
{ R: 1.0, G: 0.0, B: 0.0, A: 0.8 },
{ R: 0.0, G: 1.0, B: 0.0, A: 0.7 },
{ R: 0.0, G: 0.0, B: 0.0, A: 0.6 },
{ R: 0.0, G: 0.0, B: 0.0, A: 0.5 },
{ R: 1.0, G: 1.0, B: 1.0, A: 0.4 },
{ R: 0.7, G: 0.0, B: 0.0, A: 0.3 },
{ R: 0.0, G: 0.8, B: 0.0, A: 0.2 },
{ R: 0.0, G: 0.0, B: 0.9, A: 0.1 },
{ R: 0.1, G: 0.2, B: 0.0, A: 0.3 },
{ R: 0.4, G: 0.3, B: 0.6, A: 0.8 }];


function FloatToIntColor(c) {
  return Math.floor(c * 100);
}

const kColorsInt = kColorsFloat.map((c) => {
  return {
    R: FloatToIntColor(c.R),
    G: FloatToIntColor(c.G),
    B: FloatToIntColor(c.B),
    A: FloatToIntColor(c.A)
  };
});

const kTextureSize = 16;

function writeTextureAndGetExpectedTexelView(
t,
method,
view,
format,
sampleCount)
{
  const info = kTextureFormatInfo[format];
  const isFloatType = info.color.type === 'float' || info.color.type === 'unfilterable-float';
  const kColors = isFloatType ? kColorsFloat : kColorsInt;
  const expectedTexelView = TexelView.fromTexelsAsColors(
    format,
    (coords) => {
      const pixelPos = coords.y * kTextureSize + coords.x;
      return kColors[pixelPos % kColors.length];
    },
    { clampToFormatRange: true }
  );
  const vecType = isFloatType ? 'vec4f' : info.color.type === 'sint' ? 'vec4i' : 'vec4u';
  const kColorArrayShaderString = `array<${vecType}, ${kColors.length}>(
      ${kColors.map((t) => `${vecType}(${t.R}, ${t.G}, ${t.B}, ${t.A}) `).join(',')}
    )`;

  switch (method) {
    case 'storage-write-compute':
      {
        const pipeline = t.device.createComputePipeline({
          layout: 'auto',
          compute: {
            module: t.device.createShaderModule({
              code: `
                @group(0) @binding(0) var dst: texture_storage_2d<${format}, write>;
                @compute @workgroup_size(1, 1) fn main(
                  @builtin(global_invocation_id) global_id: vec3<u32>,
                ) {
                  const src = ${kColorArrayShaderString};
                  let coord = vec2u(global_id.xy);
                  let idx = coord.x + coord.y * ${kTextureSize};
                  textureStore(dst, coord, src[idx % ${kColors.length}]);
                }`
            }),
            entryPoint: 'main'
          }
        });
        const commandEncoder = t.device.createCommandEncoder();
        const pass = commandEncoder.beginComputePass();
        pass.setPipeline(pipeline);
        pass.setBindGroup(
          0,
          t.device.createBindGroup({
            layout: pipeline.getBindGroupLayout(0),
            entries: [
            {
              binding: 0,
              resource: view
            }]

          })
        );
        pass.dispatchWorkgroups(kTextureSize, kTextureSize);
        pass.end();
        t.device.queue.submit([commandEncoder.finish()]);
      }
      break;

    case 'storage-write-fragment':
      {
        // Create a placeholder color attachment texture,
        // The size of which equals that of format texture we are testing,
        // so that we have the same number of fragments and texels.
        const kPlaceholderTextureFormat = 'rgba8unorm';
        const placeholderTexture = t.createTextureTracked({
          format: kPlaceholderTextureFormat,
          size: [kTextureSize, kTextureSize],
          usage: GPUTextureUsage.RENDER_ATTACHMENT
        });

        const pipeline = t.device.createRenderPipeline({
          layout: 'auto',
          vertex: {
            module: t.device.createShaderModule({
              code: kFullscreenQuadVertexShaderCode
            })
          },
          fragment: {
            module: t.device.createShaderModule({
              code: `
                @group(0) @binding(0) var dst: texture_storage_2d<${format}, write>;
                @fragment fn main(
                  @builtin(position) fragCoord: vec4<f32>,
                ) {
                  const src = ${kColorArrayShaderString};
                  let coord = vec2u(fragCoord.xy);
                  let idx = coord.x + coord.y * ${kTextureSize};
                  textureStore(dst, coord, src[idx % ${kColors.length}]);
                }`
            }),
            // Set writeMask to 0 as the fragment shader has no output.
            targets: [
            {
              format: kPlaceholderTextureFormat,
              writeMask: 0
            }]

          }
        });
        const commandEncoder = t.device.createCommandEncoder();
        const pass = commandEncoder.beginRenderPass({
          colorAttachments: [
          {
            view: placeholderTexture.createView(),
            loadOp: 'clear',
            storeOp: 'discard'
          }]

        });
        pass.setPipeline(pipeline);
        pass.setBindGroup(
          0,
          t.device.createBindGroup({
            layout: pipeline.getBindGroupLayout(0),
            entries: [
            {
              binding: 0,
              resource: view
            }]

          })
        );
        pass.draw(6);
        pass.end();
        t.device.queue.submit([commandEncoder.finish()]);
      }
      break;

    case 'render-pass-store':
    case 'render-pass-resolve':
      {
        // Create a placeholder color attachment texture for the store target when tesing texture is used as resolve target.
        const targetView =
        method === 'render-pass-store' ?
        view :
        t.
        createTextureTracked({
          format,
          size: [kTextureSize, kTextureSize],
          usage: GPUTextureUsage.RENDER_ATTACHMENT,
          sampleCount: 4
        }).
        createView();
        const resolveView = method === 'render-pass-store' ? undefined : view;
        const multisampleCount = method === 'render-pass-store' ? sampleCount : 4;

        const pipeline = t.device.createRenderPipeline({
          layout: 'auto',
          vertex: {
            module: t.device.createShaderModule({
              code: kFullscreenQuadVertexShaderCode
            })
          },
          fragment: {
            module: t.device.createShaderModule({
              code: `
                @fragment fn main(
                  @builtin(position) fragCoord: vec4<f32>,
                ) -> @location(0) ${vecType} {
                  const src = ${kColorArrayShaderString};
                  let coord = vec2u(fragCoord.xy);
                  let idx = coord.x + coord.y * ${kTextureSize};
                  return src[idx % ${kColors.length}];
                }`
            }),
            targets: [
            {
              format
            }]

          },
          multisample: {
            count: multisampleCount
          }
        });
        const commandEncoder = t.device.createCommandEncoder();
        const pass = commandEncoder.beginRenderPass({
          colorAttachments: [
          {
            view: targetView,
            resolveTarget: resolveView,
            loadOp: 'clear',
            storeOp: 'store'
          }]

        });
        pass.setPipeline(pipeline);
        pass.draw(6);
        pass.end();
        t.device.queue.submit([commandEncoder.finish()]);
      }
      break;
    default:
      unreachable();
  }

  return expectedTexelView;
}

function getTextureViewUsage(
viewUsageMethod,
minimalUsageForTest)
{
  switch (viewUsageMethod) {
    case 'inherit':
      return 0;

    case 'minimal':
      return minimalUsageForTest;

    default:
      unreachable();
  }
}

g.test('format').
desc(
  `Views of every allowed format.

Read values from color array in the shader, and write it to the texture view via different write methods.

- x= every texture format
- x= sampleCount {1, 4} if valid
- x= every possible view write method (see above)
- x= inherited or minimal texture view usage

TODO: Test sampleCount > 1 for 'render-pass-store' after extending copySinglePixelTextureToBufferUsingComputePass
      to read multiple pixels from multisampled textures. [1]
TODO: Test rgb10a2uint when TexelRepresentation.numericRange is made per-component. [2]
`
).
params((u) =>
u //
.combine('method', kTextureViewWriteMethods).
combine('format', kRegularTextureFormats).
combine('sampleCount', [1, 4]).
filter(({ format, method, sampleCount }) => {
  const info = kTextureFormatInfo[format];

  if (sampleCount > 1 && !info.multisample) {
    return false;
  }

  // [2]
  if (format === 'rgb10a2uint') {
    return false;
  }

  switch (method) {
    case 'storage-write-compute':
    case 'storage-write-fragment':
      return info.color?.storage && sampleCount === 1;
    case 'render-pass-store':
      // [1]
      if (sampleCount > 1) {
        return false;
      }
      return !!info.colorRender;
    case 'render-pass-resolve':
      return !!info.colorRender?.resolve && sampleCount === 1;
  }
  return true;
}).
combine('viewUsageMethod', kTextureViewUsageMethods)
).
beforeAllSubcases((t) => {
  const { format, method } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  switch (method) {
    case 'storage-write-compute':
    case 'storage-write-fragment':
      // Still need to filter again for compat mode.
      t.skipIfTextureFormatNotUsableAsStorageTexture(format);
      break;
  }
}).
fn((t) => {
  const { format, method, sampleCount, viewUsageMethod } = t.params;

  const textureUsageForMethod = method.includes('storage') ?
  GPUTextureUsage.STORAGE_BINDING :
  GPUTextureUsage.RENDER_ATTACHMENT;
  const usage = GPUTextureUsage.COPY_SRC | textureUsageForMethod;

  const texture = t.createTextureTracked({
    format,
    usage,
    size: [kTextureSize, kTextureSize],
    sampleCount
  });

  const view = texture.createView({
    usage: getTextureViewUsage(viewUsageMethod, textureUsageForMethod)
  });
  const expectedTexelView = writeTextureAndGetExpectedTexelView(
    t,
    method,
    view,
    format,
    sampleCount
  );

  // [1] Use copySinglePixelTextureToBufferUsingComputePass to check multisampled texture.
  t.expectTexelViewComparisonIsOkInTexture({ texture }, expectedTexelView, [
  kTextureSize,
  kTextureSize]
  );
});

g.test('dimension').
desc(
  `Views of every allowed dimension.

- x= a representative subset of formats
- x= {every texture dimension} x {every valid view dimension}
  (per gpuweb#79 no dimension-count reinterpretations, like 2d-array <-> 3d, are possible)
- x= sampleCount {1, 4} if valid
- x= every possible view write method (see above)
`
).
unimplemented();

g.test('aspect').
desc(
  `Views of every allowed aspect of depth/stencil textures.

- x= every depth/stencil format
- x= {"all", "stencil-only", "depth-only"} where valid for the format
- x= sampleCount {1, 4} if valid
- x= every possible view write method (see above)
`
).
unimplemented();