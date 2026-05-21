/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `API Operation Tests for multisample resolve in render passes.`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { isTextureFormatPossiblyResolvable, kColorTextureFormats } from '../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';
import * as ttu from '../../../texture_test_utils.js';

const kSlotsToResolve = [
[0, 2],
[1, 3],
[0, 1, 2, 3]];


const kSize = 4;
const kDepthStencilFormat = 'depth24plus-stencil8';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('render_pass_resolve').
desc(
  `
Test basic render pass resolve behavior for combinations of:
  - number of color attachments, some with and some without a resolveTarget
  - {a single draw+resolve pass, one draw-store pass and one empty load-resolve pass}
    (attempts to test known driver bugs with empty resolve passes)
  - in the resolve pass, storeOp set to {'store', 'discard'}
  - mip levels {0, 1} and array layers {0, 1}
    TODO: cases where color attachment and resolve target don't have the same mip level
  - resolveTarget {2d array layer, TODO: 3d slice} {0, >0} with {2d, TODO: 3d} resolveTarget
    TODO: cases where color attachment and resolve target don't have the same z (slice or layer)
  - TODO: test that any not-resolved attachments are rendered to correctly.
  - TODO: test different loadOps
  - TODO?: resolveTarget mip level {0, >0} (TODO?: different mip level from colorAttachment)
  - TODO?: resolveTarget {2d array layer, TODO: 3d slice} {0, >0} with {2d, TODO: 3d} resolveTarget
    (different z from colorAttachment)
`
).
params((u) =>
u.
combine('colorFormat', kColorTextureFormats).
filter((t) => isTextureFormatPossiblyResolvable(t.colorFormat)).
combine('separateResolvePass', [false, true]).
combine('storeOperation', ['discard', 'store']).
beginSubcases().
combine('transientColorAttachment', [false, true])
// Exclude if transient AND (separate pass OR storing)
.unless(
  (t) => t.transientColorAttachment && (t.separateResolvePass || t.storeOperation === 'store')
).
combine('numColorAttachments', [2, 4]).
combine('slotsToResolve', kSlotsToResolve).
combine('resolveTargetBaseMipLevel', [0, 1]).
combine('resolveTargetBaseArrayLayer', [0, 1]).
combine('transientDepthStencilAttachment', [false, true]).
combine('depthStencilAttachment', [false, true]).
unless((t) => !t.depthStencilAttachment && t.transientDepthStencilAttachment)
).
fn((t) => {
  const { colorFormat } = t.params;
  t.skipIfTextureFormatNotSupported(colorFormat);
  t.skipIfTextureFormatNotResolvable(colorFormat);
  // MAINTENANCE_TODO(#4509): Remove this after all implementations have TRANSIENT_ATTACHMENT.
  if (t.params.transientColorAttachment || t.params.transientDepthStencilAttachment) {
    t.skipIfTransientAttachmentNotSupported();
  }

  const targets = [];
  for (let i = 0; i < t.params.numColorAttachments; i++) {
    targets.push({ format: colorFormat });
  }

  let depthStencil;
  if (t.params.depthStencilAttachment) {
    depthStencil = {
      format: kDepthStencilFormat,
      depthWriteEnabled: false
    };
  }

  // These shaders will draw a white triangle into a texture. After draw, the top left
  // half of the texture will be white, and the bottom right half will be unchanged. When this
  // texture is resolved, there will be two distinct colors in each portion of the texture, as
  // well as a line between the portions that contain the midpoint color due to the multisample
  // resolve.
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: `
            @vertex fn main(
              @builtin(vertex_index) VertexIndex : u32
              ) -> @builtin(position) vec4<f32> {
              var pos : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
                  vec2<f32>(-1.0, -1.0),
                  vec2<f32>(-1.0,  1.0),
                  vec2<f32>( 1.0,  1.0));
              return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
            }`
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
            struct Output {
              @location(0) fragColor0 : vec4<f32>,
              @location(1) fragColor1 : vec4<f32>,
              @location(2) fragColor2 : vec4<f32>,
              @location(3) fragColor3 : vec4<f32>,
            };

            @fragment fn main() -> Output {
              return Output(
                vec4<f32>(1.0, 1.0, 1.0, 1.0),
                vec4<f32>(1.0, 1.0, 1.0, 1.0),
                vec4<f32>(1.0, 1.0, 1.0, 1.0),
                vec4<f32>(1.0, 1.0, 1.0, 1.0)
              );
            }`
      }),
      entryPoint: 'main',
      targets
    },
    depthStencil,
    primitive: { topology: 'triangle-list' },
    multisample: { count: 4 }
  });

  const resolveTargets = [];
  const drawPassAttachments = [];
  const resolvePassAttachments = [];

  // The resolve target must be the same size as the color attachment. If we're resolving to mip
  // level 1, the resolve target base mip level should be 2x the color attachment size.
  const kResolveTargetSize = kSize << t.params.resolveTargetBaseMipLevel;

  for (let i = 0; i < t.params.numColorAttachments; i++) {
    const colorAttachment = t.
    createTextureTracked({
      format: colorFormat,
      size: [kSize, kSize, 1],
      sampleCount: 4,
      mipLevelCount: 1,
      usage: t.params.transientColorAttachment ?
      GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TRANSIENT_ATTACHMENT :
      GPUTextureUsage.RENDER_ATTACHMENT
    }).
    createView();

    let resolveTarget;
    if (t.params.slotsToResolve.includes(i)) {
      const resolveTargetTexture = t.createTextureTracked({
        format: colorFormat,
        size: [kResolveTargetSize, kResolveTargetSize, t.params.resolveTargetBaseArrayLayer + 1],
        sampleCount: 1,
        mipLevelCount: t.params.resolveTargetBaseMipLevel + 1,
        usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
      });
      resolveTargets.push(resolveTargetTexture);

      resolveTarget = resolveTargetTexture.createView({
        baseMipLevel: t.params.resolveTargetBaseMipLevel,
        baseArrayLayer: t.params.resolveTargetBaseArrayLayer
      });
    }

    // Clear to black for the load operation. After the draw, the top left half of the attachment
    // will be white and the bottom right half will be black.
    if (t.params.separateResolvePass) {
      drawPassAttachments.push({
        view: colorAttachment,
        clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
        loadOp: 'clear',
        storeOp: 'store'
      });
      resolvePassAttachments.push({
        view: colorAttachment,
        resolveTarget,
        loadOp: 'load',
        storeOp: t.params.storeOperation
      });
    } else {
      drawPassAttachments.push({
        view: colorAttachment,
        resolveTarget,
        clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
        loadOp: 'clear',
        storeOp: t.params.storeOperation
      });
    }
  }

  let depthStencilAttachment;
  if (t.params.depthStencilAttachment) {
    const depthStencilTexture = t.createTextureTracked({
      format: kDepthStencilFormat,
      size: [kSize, kSize, 1],
      sampleCount: 4,
      usage: t.params.transientDepthStencilAttachment ?
      GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TRANSIENT_ATTACHMENT :
      GPUTextureUsage.RENDER_ATTACHMENT
    });
    depthStencilAttachment = {
      view: depthStencilTexture.createView(),
      depthClearValue: 1.0,
      depthLoadOp: 'clear',
      depthStoreOp: 'discard',
      stencilLoadOp: 'clear',
      stencilStoreOp: 'discard'
    };
  }

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: drawPassAttachments,
    depthStencilAttachment
  });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();
  if (t.params.separateResolvePass) {
    const pass = encoder.beginRenderPass({
      colorAttachments: resolvePassAttachments,
      depthStencilAttachment
    });
    pass.end();
  }
  t.device.queue.submit([encoder.finish()]);

  // Verify the resolve targets contain the correct values. Note that we use z to specify the
  // array layer from which to pull the pixels for testing.
  const z = t.params.resolveTargetBaseArrayLayer;
  for (const resolveTarget of resolveTargets) {
    ttu.expectSinglePixelComparisonsAreOkInTexture(
      t,
      { texture: resolveTarget, mipLevel: t.params.resolveTargetBaseMipLevel },
      // Note: Channels that do not exist in the actual texture are not compared.
      [
      // Top left pixel should be {1.0, 1.0, 1.0, 1.0}.
      { coord: { x: 0, y: 0, z }, exp: { R: 1.0, G: 1.0, B: 1.0, A: 1.0 } },
      // Bottom right pixel should be {0, 0, 0, 0}.
      { coord: { x: kSize - 1, y: kSize - 1, z }, exp: { R: 0, G: 0, B: 0, A: 0 } },
      // Top right pixel should be {0.5, 0.5, 0.5, 0.5} due to the multisampled resolve.
      { coord: { x: kSize - 1, y: 0, z }, exp: { R: 0.5, G: 0.5, B: 0.5, A: 0.5 } }]

    );
  }
});