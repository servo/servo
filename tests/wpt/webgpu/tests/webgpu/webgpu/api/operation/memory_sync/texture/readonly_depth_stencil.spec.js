/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Memory synchronization tests for depth-stencil attachments in a single pass, with checks for readonlyness.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { kDepthStencilFormats, kTextureFormatInfo } from '../../../../format_info.js';
import { GPUTest } from '../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('sampling_while_testing').
desc(
  `Tests concurrent sampling and testing of readonly depth-stencil attachments in a render pass.
      - Test for all depth-stencil formats.
      - Test for all valid combinations of depth/stencilReadOnly.

In particular this test checks that a non-readonly aspect can be rendered to, and used for depth/stencil
testing while the other one is used for sampling.
  `
).
params((p) =>
p.
combine('format', kDepthStencilFormats) //
.combine('depthReadOnly', [true, false, undefined]).
combine('stencilReadOnly', [true, false, undefined]).
filter((p) => {
  const info = kTextureFormatInfo[p.format];
  const depthMatch = info.depth === undefined === (p.depthReadOnly === undefined);
  const stencilMatch = info.stencil === undefined === (p.stencilReadOnly === undefined);
  return depthMatch && stencilMatch;
})
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  const formatInfo = kTextureFormatInfo[format];
  const hasDepth = formatInfo.depth !== undefined;
  const hasStencil = formatInfo.stencil !== undefined;

  t.selectDeviceForTextureFormatOrSkipTestCase(t.params.format);
  t.skipIf(
    t.isCompatibility && hasDepth && hasStencil,
    'compatibility mode does not support different TEXTURE_BINDING views of the same texture in a single draw calls'
  );
}).
fn((t) => {
  const { format, depthReadOnly, stencilReadOnly } = t.params;
  const formatInfo = kTextureFormatInfo[format];
  const hasDepth = formatInfo.depth !== undefined;
  const hasStencil = formatInfo.stencil !== undefined;

  // The 3x3 depth stencil texture used for the tests.
  const ds = t.createTextureTracked({
    label: 'testTexture',
    size: [3, 3],
    format,
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING
  });

  // Fill the texture along the X axis with stencil values 1, 2, 3 and along the Y axis depth
  // values 0.1, 0.2, 0.3. The depth value is written using @builtin(frag_depth) while the
  // stencil is written using stencil operation and modifying the stencilReference.
  const initModule = t.device.createShaderModule({
    code: `
            @vertex fn vs(
                @builtin(instance_index) x : u32, @builtin(vertex_index) y : u32
            ) -> @builtin(position) vec4f {
                let texcoord = (vec2f(f32(x), f32(y)) + vec2f(0.5)) / 3;
                return vec4f((texcoord * 2) - vec2f(1.0), 0, 1);
            }
            @fragment fn fs_with_depth(@builtin(position) pos : vec4f) -> @builtin(frag_depth) f32 {
                return (pos.y + 0.5) / 10;
            }
            @fragment fn fs_no_depth() {
            }
        `
  });
  const initPipeline = t.device.createRenderPipeline({
    layout: 'auto',
    label: 'initPipeline',
    vertex: { module: initModule },
    fragment: {
      module: initModule,
      targets: [],
      entryPoint: hasDepth ? 'fs_with_depth' : 'fs_no_depth'
    },
    depthStencil: {
      format,
      ...(hasDepth && {
        depthWriteEnabled: true,
        depthCompare: 'always'
      }),
      ...(hasStencil && {
        stencilBack: { compare: 'always', passOp: 'replace' },
        stencilFront: { compare: 'always', passOp: 'replace' }
      })
    },
    primitive: { topology: 'point-list' }
  });

  const encoder = t.device.createCommandEncoder();

  const initPass = encoder.beginRenderPass({
    colorAttachments: [],
    depthStencilAttachment: {
      view: ds.createView(),
      ...(hasDepth && {
        depthLoadOp: 'clear',
        depthStoreOp: 'store',
        depthClearValue: 0
      }),
      ...(hasStencil && {
        stencilLoadOp: 'clear',
        stencilStoreOp: 'store',
        stencilClearValue: 0
      })
    }
  });
  initPass.setPipeline(initPipeline);
  for (let i = 0; i < 3; i++) {
    initPass.setStencilReference(i + 1);
    // Draw 3 points (Y = 0, 1, 2) at X = instance_index = i.
    initPass.draw(3, 1, 0, i);
  }
  initPass.end();

  // Perform the actual test:
  //   - The shader outputs depth 0.15 and stencil 2 (via stencilReference).
  //   - Test that the fragdepth / stencilref must be <= to what's in the depth-stencil attachment.
  //      -> Fragments that have depth 0.1 or stencil 1 are tested out.
  //   - Test that sampling the depth / stencil (when possible) is <= 0.2 for depth, <= 2 for stencil
  //      -> Fragments that have depth 0.3 or stencil 3 are discarded if that aspect is readonly.
  //   - Write the depth / increment the stencil if the aspect is not readonly.
  //      -> After the test, fragments that passed will have non-readonly aspects updated.
  const kFragDepth = 0.15;
  const kStencilRef = 2;
  const testAndCheckModule = t.device.createShaderModule({
    code: `
          @group(0) @binding(0) var depthTex : texture_2d<f32>;
          @group(0) @binding(1) var stencilTex : texture_2d<u32>;

          @vertex fn full_quad_vs(@builtin(vertex_index) id : u32) -> @builtin(position) vec4f {
            let pos = array(vec2f(-3, -1), vec2(3, -1), vec2(0, 2));
            return vec4f(pos[id], ${kFragDepth}, 1.0);
          }

          @fragment fn test_texture(@builtin(position) pos : vec4f) {
            let texel = vec2u(floor(pos.xy));
            if ${!!stencilReadOnly} && textureLoad(stencilTex, texel, 0).r > 2 {
                discard;
            }
            if ${!!depthReadOnly} && textureLoad(depthTex, texel, 0).r > 0.21 {
                discard;
            }
          }

          @fragment fn check_texture(@builtin(position) pos : vec4f) -> @location(0) u32 {
            let texel = vec2u(floor(pos.xy));

            // The current values in the framebuffer.
            let initStencil = texel.x + 1;
            let initDepth = f32(texel.y + 1) / 10.0;

            // Expected results of the test_texture step.
            let stencilTestPasses = !${hasStencil} || ${kStencilRef} <= initStencil;
            let depthTestPasses = !${hasDepth} || ${kFragDepth} <= initDepth;
            let fsDiscards = (${!!stencilReadOnly} && initStencil > 2) ||
                             (${!!depthReadOnly} && initDepth > 0.21);

            // Compute the values that should be in the framebuffer.
            var stencil = initStencil;
            var depth = initDepth;

            // When the fragments aren't discarded, fragment output operations happen.
            if depthTestPasses && stencilTestPasses && !fsDiscards {
                if ${!stencilReadOnly} {
                    stencil += 1;
                }
                if ${!depthReadOnly} {
                    depth = ${kFragDepth};
                }
            }

            if ${hasStencil} && textureLoad(stencilTex, texel, 0).r != stencil {
                return 0;
            }
            if ${hasDepth} && abs(textureLoad(depthTex, texel, 0).r - depth) > 0.01 {
                return 0;
            }
            return 1;
          }
    `
  });
  const testPipeline = t.device.createRenderPipeline({
    label: 'testPipeline',
    layout: 'auto',
    vertex: { module: testAndCheckModule },
    fragment: { module: testAndCheckModule, entryPoint: 'test_texture', targets: [] },
    depthStencil: {
      format,
      ...(hasDepth && {
        depthCompare: 'less-equal',
        depthWriteEnabled: !depthReadOnly
      }),
      ...(hasStencil && {
        stencilBack: {
          compare: 'less-equal',
          passOp: stencilReadOnly ? 'keep' : 'increment-clamp'
        },
        stencilFront: {
          compare: 'less-equal',
          passOp: stencilReadOnly ? 'keep' : 'increment-clamp'
        }
      })
    },
    primitive: { topology: 'triangle-list' }
  });

  // Make fake stencil or depth textures to put in the bindgroup if the aspect is not readonly.
  const fakeStencil = t.createTextureTracked({
    label: 'fakeStencil',
    format: 'r32uint',
    size: [1, 1],
    usage: GPUTextureUsage.TEXTURE_BINDING
  });
  const fakeDepth = t.createTextureTracked({
    label: 'fakeDepth',
    format: 'r32float',
    size: [1, 1],
    usage: GPUTextureUsage.TEXTURE_BINDING
  });
  const stencilView = stencilReadOnly ?
  ds.createView({ aspect: 'stencil-only' }) :
  fakeStencil.createView();
  const depthView = depthReadOnly ?
  ds.createView({ aspect: 'depth-only' }) :
  fakeDepth.createView();
  const testBindGroup = t.device.createBindGroup({
    layout: testPipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: depthView },
    { binding: 1, resource: stencilView }]

  });

  // Run the test.
  const testPass = encoder.beginRenderPass({
    colorAttachments: [],
    depthStencilAttachment: {
      view: ds.createView(),
      ...(hasDepth && (
      depthReadOnly ?
      { depthReadOnly: true } :
      {
        depthLoadOp: 'load',
        depthStoreOp: 'store'
      })),
      ...(hasStencil && (
      stencilReadOnly ?
      { stencilReadOnly: true } :
      {
        stencilLoadOp: 'load',
        stencilStoreOp: 'store'
      }))
    }
  });
  testPass.setPipeline(testPipeline);
  testPass.setStencilReference(kStencilRef);
  testPass.setBindGroup(0, testBindGroup);
  testPass.draw(3);
  testPass.end();

  // Check that the contents of the textures are what we expect. See the shader module for the
  // computation of what's expected, it writes a 1 on success, 0 otherwise.
  const checkPipeline = t.device.createRenderPipeline({
    label: 'checkPipeline',
    layout: 'auto',
    vertex: { module: testAndCheckModule },
    fragment: {
      module: testAndCheckModule,
      entryPoint: 'check_texture',
      targets: [{ format: 'r32uint' }]
    },
    primitive: { topology: 'triangle-list' }
  });
  const checkBindGroup = t.device.createBindGroup({
    layout: checkPipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: hasDepth ? ds.createView({ aspect: 'depth-only' }) : fakeDepth.createView()
    },
    {
      binding: 1,
      resource: hasStencil ?
      ds.createView({ aspect: 'stencil-only' }) :
      fakeStencil.createView()
    }]

  });

  const resultTexture = t.createTextureTracked({
    label: 'resultTexture',
    format: 'r32uint',
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
    size: [3, 3]
  });
  const checkPass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: resultTexture.createView(),
      loadOp: 'clear',
      clearValue: [0, 0, 0, 0],
      storeOp: 'store'
    }]

  });
  checkPass.setPipeline(checkPipeline);
  checkPass.setBindGroup(0, checkBindGroup);
  checkPass.draw(3);
  checkPass.end();

  t.queue.submit([encoder.finish()]);

  // The check texture should be full of success (a.k.a. 1)!
  t.expectSingleColor(resultTexture, resultTexture.format, { size: [3, 3, 1], exp: { R: 1 } });
});