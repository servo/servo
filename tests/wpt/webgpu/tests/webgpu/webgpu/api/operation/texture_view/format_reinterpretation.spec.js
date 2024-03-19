/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test texture views can reinterpret the format of the original texture.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import {
  kRenderableColorTextureFormats,
  kRegularTextureFormats,
  viewCompatible } from

'../../../format_info.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';
import { TexelView } from '../../../util/texture/texel_view.js';

export const g = makeTestGroup(TextureTestMixin(GPUTest));

const kColors = [
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


const kTextureSize = 16;

function makeInputTexelView(format) {
  return TexelView.fromTexelsAsColors(
    format,
    (coords) => {
      const pixelPos = coords.y * kTextureSize + coords.x;
      return kColors[pixelPos % kColors.length];
    },
    { clampToFormatRange: true }
  );
}

function makeBlitPipeline(
device,
format,
multisample)
{
  return device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: device.createShaderModule({
        code: `
          @vertex fn main(@builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4<f32> {
            var pos = array<vec2<f32>, 6>(
                                        vec2<f32>(-1.0, -1.0),
                                        vec2<f32>(-1.0,  1.0),
                                        vec2<f32>( 1.0, -1.0),
                                        vec2<f32>(-1.0,  1.0),
                                        vec2<f32>( 1.0, -1.0),
                                        vec2<f32>( 1.0,  1.0));
            return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
          }`
      }),
      entryPoint: 'main'
    },
    fragment: {
      module:
      multisample.sample > 1 ?
      device.createShaderModule({
        code: `
            @group(0) @binding(0) var src: texture_multisampled_2d<f32>;
            @fragment fn main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
              var result : vec4<f32>;
              for (var i = 0; i < ${multisample.sample}; i = i + 1) {
                result = result + textureLoad(src, vec2<i32>(coord.xy), i);
              }
              return result * ${1 / multisample.sample};
            }`
      }) :
      device.createShaderModule({
        code: `
            @group(0) @binding(0) var src: texture_2d<f32>;
            @fragment fn main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
              return textureLoad(src, vec2<i32>(coord.xy), 0);
            }`
      }),
      entryPoint: 'main',
      targets: [{ format }]
    },
    multisample: {
      count: multisample.render
    }
  });
}

g.test('texture_binding').
desc(`Test that a regular texture allocated as 'format' is correctly sampled as 'viewFormat'.`).
params((u) =>
u //
.combine('format', kRegularTextureFormats).
combine('viewFormat', kRegularTextureFormats).
filter(
  ({ format, viewFormat }) =>
  format !== viewFormat && viewCompatible(false, format, viewFormat)
)
).
beforeAllSubcases((t) => {
  const { format, viewFormat } = t.params;
  t.skipIfTextureFormatNotSupported(format, viewFormat);
  // Compatibility mode does not support format reinterpretation.
  t.skipIf(t.isCompatibility);
}).
fn((t) => {
  const { format, viewFormat } = t.params;

  // Make an input texel view.
  const inputTexelView = makeInputTexelView(format);

  // Create the initial texture with the contents if the input texel view.
  const texture = t.createTextureFromTexelView(inputTexelView, {
    size: [kTextureSize, kTextureSize],
    usage: GPUTextureUsage.TEXTURE_BINDING,
    viewFormats: [viewFormat]
  });

  // Reinterpret the texture as the view format.
  // Make a texel view of the format that also reinterprets the data.
  const reinterpretedView = texture.createView({ format: viewFormat });
  const reinterpretedTexelView = TexelView.fromTexelsAsBytes(viewFormat, inputTexelView.bytes);

  // Create a pipeline to write data out to rgba8unorm.
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
          @group(0) @binding(0) var src: texture_2d<f32>;
          @group(0) @binding(1) var dst: texture_storage_2d<rgba8unorm, write>;
          @compute @workgroup_size(1, 1) fn main(
            @builtin(global_invocation_id) global_id: vec3<u32>,
          ) {
            var coord = vec2<i32>(global_id.xy);
            textureStore(dst, coord, textureLoad(src, coord, 0));
          }`
      }),
      entryPoint: 'main'
    }
  });

  // Create an rgba8unorm output texture.
  const outputTexture = t.trackForCleanup(
    t.device.createTexture({
      format: 'rgba8unorm',
      size: [kTextureSize, kTextureSize],
      usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.COPY_SRC
    })
  );

  // Execute a compute pass to load data from the reinterpreted view and
  // write out to the rgba8unorm texture.
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
        resource: reinterpretedView
      },
      {
        binding: 1,
        resource: outputTexture.createView()
      }]

    })
  );
  pass.dispatchWorkgroups(kTextureSize, kTextureSize);
  pass.end();
  t.device.queue.submit([commandEncoder.finish()]);

  t.expectTexelViewComparisonIsOkInTexture(
    { texture: outputTexture },
    TexelView.fromTexelsAsColors('rgba8unorm', reinterpretedTexelView.color, {
      clampToFormatRange: true
    }),
    [kTextureSize, kTextureSize]
  );
});

g.test('render_and_resolve_attachment').
desc(
  `Test that a color render attachment allocated as 'format' is correctly rendered to as 'viewFormat',
and resolved to an attachment allocated as 'format' viewed as 'viewFormat'.

Other combinations aren't possible because the render and resolve targets must both match
in view format and match in base format.`
).
params((u) =>
u //
.combine('format', kRenderableColorTextureFormats).
combine('viewFormat', kRenderableColorTextureFormats).
filter(
  ({ format, viewFormat }) =>
  format !== viewFormat && viewCompatible(false, format, viewFormat)
).
combine('sampleCount', [1, 4])
).
beforeAllSubcases((t) => {
  const { format, viewFormat } = t.params;
  t.skipIfTextureFormatNotSupported(format, viewFormat);
  // Compatibility mode does not support format reinterpretation.
  t.skipIf(t.isCompatibility);
}).
fn((t) => {
  const { format, viewFormat, sampleCount } = t.params;

  // Make an input texel view.
  const inputTexelView = makeInputTexelView(format);

  // Create the renderTexture as |format|.
  const renderTexture = t.trackForCleanup(
    t.device.createTexture({
      format,
      size: [kTextureSize, kTextureSize],
      usage:
      GPUTextureUsage.RENDER_ATTACHMENT | (
      sampleCount > 1 ? GPUTextureUsage.TEXTURE_BINDING : GPUTextureUsage.COPY_SRC),
      viewFormats: [viewFormat],
      sampleCount
    })
  );

  const resolveTexture =
  sampleCount === 1 ?
  undefined :
  t.trackForCleanup(
    t.device.createTexture({
      format,
      size: [kTextureSize, kTextureSize],
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
      viewFormats: [viewFormat]
    })
  );

  // Create the sample source with the contents of the input texel view.
  // We will sample this texture into |renderTexture|. It uses the same format to keep the same
  // number of bits of precision.
  const sampleSource = t.createTextureFromTexelView(inputTexelView, {
    size: [kTextureSize, kTextureSize],
    usage: GPUTextureUsage.TEXTURE_BINDING
  });

  // Reinterpret the renderTexture as |viewFormat|.
  const reinterpretedRenderView = renderTexture.createView({ format: viewFormat });
  const reinterpretedResolveView =
  resolveTexture && resolveTexture.createView({ format: viewFormat });

  // Create a pipeline to blit a src texture to the render attachment.
  const pipeline = makeBlitPipeline(t.device, viewFormat, {
    sample: 1,
    render: sampleCount
  });

  // Execute a render pass to sample |sampleSource| into |texture| viewed as |viewFormat|.
  const commandEncoder = t.device.createCommandEncoder();
  const pass = commandEncoder.beginRenderPass({
    colorAttachments: [
    {
      view: reinterpretedRenderView,
      resolveTarget: reinterpretedResolveView,
      loadOp: 'load',
      storeOp: 'store'
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
        resource: sampleSource.createView()
      }]

    })
  );
  pass.draw(6);
  pass.end();

  // If the render target is multisampled, we'll manually resolve it to check
  // the contents.
  const singleSampleRenderTexture = resolveTexture ?
  t.trackForCleanup(
    t.device.createTexture({
      format,
      size: [kTextureSize, kTextureSize],
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
    })
  ) :
  renderTexture;

  if (resolveTexture) {
    // Create a pipeline to blit the multisampled render texture to a non-multisample texture.
    // We are basically performing a manual resolve step to the same format as the original
    // render texture to check its contents.
    const pipeline = makeBlitPipeline(t.device, format, { sample: sampleCount, render: 1 });
    const pass = commandEncoder.beginRenderPass({
      colorAttachments: [
      {
        view: singleSampleRenderTexture.createView(),
        loadOp: 'load',
        storeOp: 'store'
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
          resource: renderTexture.createView()
        }]

      })
    );
    pass.draw(6);
    pass.end();
  }

  // Submit the commands.
  t.device.queue.submit([commandEncoder.finish()]);

  // Check the rendered contents.
  const renderViewTexels = TexelView.fromTexelsAsColors(viewFormat, inputTexelView.color, {
    clampToFormatRange: true
  });
  t.expectTexelViewComparisonIsOkInTexture(
    { texture: singleSampleRenderTexture },
    renderViewTexels,
    [kTextureSize, kTextureSize],
    { maxDiffULPsForNormFormat: 2 }
  );

  // Check the resolved contents.
  if (resolveTexture) {
    const resolveView = TexelView.fromTexelsAsColors(viewFormat, renderViewTexels.color, {
      clampToFormatRange: true
    });
    t.expectTexelViewComparisonIsOkInTexture(
      { texture: resolveTexture },
      resolveView,
      [kTextureSize, kTextureSize],
      { maxDiffULPsForNormFormat: 2 }
    );
  }
});