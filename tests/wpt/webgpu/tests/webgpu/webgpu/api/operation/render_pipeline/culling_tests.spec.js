/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Test culling and rasterization state.

Test coverage:
Test all culling combinations of GPUFrontFace and GPUCullMode show the correct output.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kTextureFormatInfo } from '../../../format_info.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';

function faceIsCulled(face, frontFace, cullMode) {
  return cullMode !== 'none' && frontFace === face === (cullMode === 'front');
}

function faceColor(face, frontFace, cullMode) {
  // front facing color is green, non front facing is red, background is blue
  const isCulled = faceIsCulled(face, frontFace, cullMode);
  if (!isCulled && face === frontFace) {
    return new Uint8Array([0x00, 0xff, 0x00, 0xff]);
  } else if (isCulled) {
    return new Uint8Array([0x00, 0x00, 0xff, 0xff]);
  } else {
    return new Uint8Array([0xff, 0x00, 0x00, 0xff]);
  }
}

class CullingTest extends TextureTestMixin(GPUTest) {
  checkCornerPixels(
  texture,
  expectedTopLeftColor,
  expectedBottomRightColor)
  {
    this.expectSinglePixelComparisonsAreOkInTexture({ texture }, [
    { coord: { x: 0, y: 0 }, exp: expectedTopLeftColor },
    { coord: { x: texture.width - 1, y: texture.height - 1 }, exp: expectedBottomRightColor }]
    );
  }

  drawFullClipSpaceTriangleAndCheckCornerPixels(
  texture,
  format,
  topology,
  color,
  depthStencil,
  depthStencilAttachment,
  expectedTopLeftColor,
  expectedBottomRightColor)
  {
    const { device } = this;
    const encoder = device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: texture.createView(),
        loadOp: 'load',
        storeOp: 'store'
      }],

      depthStencilAttachment
    });

    pass.setPipeline(
      device.createRenderPipeline({
        layout: 'auto',
        vertex: {
          module: device.createShaderModule({
            code: `
              @vertex fn main(
                @builtin(vertex_index) VertexIndex : u32
                ) -> @builtin(position) vec4<f32> {
                  var pos : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
                    vec2<f32>(-1.0, -1.0),
                    vec2<f32>( 3.0, -1.0),
                    vec2<f32>(-1.0,  3.0));
                return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
              }`
          }),
          entryPoint: 'main'
        },
        fragment: {
          module: device.createShaderModule({
            code: `
              @fragment fn main() -> @location(0) vec4<f32> {
                return vec4f(${Array.from(color).map((v) => v / 255)});
              }`
          }),
          entryPoint: 'main',
          targets: [{ format }]
        },
        primitive: {
          topology
        },
        depthStencil
      })
    );
    pass.draw(3);
    pass.end();

    device.queue.submit([encoder.finish()]);

    this.checkCornerPixels(texture, expectedTopLeftColor, expectedBottomRightColor);
  }
}

export const g = makeTestGroup(CullingTest);

g.test('culling').
desc(
  `
    Test 2 triangles with different winding orders:

    - Test that the counter-clock wise triangle has correct output for:
      - All FrontFaces (ccw, cw)
      - All CullModes (none, front, back)
      - All depth stencil attachment types (none, depth24plus, depth32float, depth24plus-stencil8)
      - Some primitive topologies (triangle-list, triangle-strip)

    - Test that the clock wise triangle has correct output for:
      - All FrontFaces (ccw, cw)
      - All CullModes (none, front, back)
      - All depth stencil attachment types (none, depth24plus, depth32float, depth24plus-stencil8)
      - Some primitive topologies (triangle-list, triangle-strip)
    `
).
params((u) =>
u.
combine('frontFace', ['ccw', 'cw']).
combine('cullMode', ['none', 'front', 'back']).
beginSubcases().
combine('depthStencilFormat', [
null,
'depth24plus',
'depth32float',
'depth24plus-stencil8']
).
combine('topology', ['triangle-list', 'triangle-strip'])
).
fn((t) => {
  const { frontFace, cullMode, depthStencilFormat, topology } = t.params;
  const size = 4;
  const format = 'rgba8unorm';

  const texture = t.device.createTexture({
    size: { width: size, height: size, depthOrArrayLayers: 1 },
    format,
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  });

  const haveStencil = depthStencilFormat && kTextureFormatInfo[depthStencilFormat].stencil;
  let depthTexture = undefined;
  let depthStencilAttachment = undefined;
  let depthStencil = undefined;
  if (depthStencilFormat) {
    depthTexture = t.device.createTexture({
      size: { width: size, height: size, depthOrArrayLayers: 1 },
      format: depthStencilFormat,
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    });

    depthStencilAttachment = {
      view: depthTexture.createView(),
      depthClearValue: 1.0,
      depthLoadOp: 'clear',
      depthStoreOp: 'store'
    };

    depthStencil = {
      format: depthStencilFormat,
      depthCompare: 'less',
      depthWriteEnabled: true
    };

    if (haveStencil) {
      depthStencilAttachment.stencilLoadOp = 'clear';
      depthStencilAttachment.stencilStoreOp = 'store';
      depthStencil.stencilFront = { passOp: 'increment-clamp' };
      depthStencil.stencilBack = { passOp: 'increment-clamp' };
    }
  }

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: texture.createView(),
      clearValue: [0, 0, 1, 1],
      loadOp: 'clear',
      storeOp: 'store'
    }],

    depthStencilAttachment
  });

  // Draw triangles with different winding orders:
  //
  // for triangle-list, 2 triangles
  //   1. The top-left one is counterclockwise (CCW)
  //   2. The bottom-right one is clockwise (CW)
  //
  //     0---2---+
  //     |   |   |
  //     |   |   |
  //     1---+---4
  //     |   |   |
  //     |   |   |
  //     +---3---5
  //
  // for triangle-strip, 4 triangles
  // note: for triangle-strip the index order swaps every other triangle
  // so the order is 012, 213, 234, 435
  //
  //   1. The top left is counterclockwise (CCW)
  //   2. zero size
  //   3. zero size
  //   4. The bottom right one is clockwise (CW)
  //
  //         0
  //         |
  //         |
  //     +---+---+
  //     |   |   |
  //     |   |   |
  // 1---+---23--+---5
  //     |   |   |
  //     |   |   |
  //     +---+---+
  //         |
  //         |
  //         4
  pass.setPipeline(
    t.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: t.device.createShaderModule({
          code: `
              @vertex fn main(
                @builtin(vertex_index) VertexIndex : u32
                ) -> @builtin(position) vec4<f32> {
                  var pos : array<vec2<f32>, 6> = array<vec2<f32>, 6>(
                ${
          topology === 'triangle-list' ?
          `
                    vec2<f32>(-1.0,  1.0),
                    vec2<f32>(-1.0,  0.0),
                    vec2<f32>( 0.0,  1.0),
                    vec2<f32>( 0.0, -1.0),
                    vec2<f32>( 1.0,  0.0),
                    vec2<f32>( 1.0, -1.0));
                ` :
          `
                    vec2<f32>( 0.0,  2.0),
                    vec2<f32>(-2.0,  0.0),
                    vec2<f32>( 0.0,  0.0),
                    vec2<f32>( 0.0,  0.0),
                    vec2<f32>( 0.0, -2.0),
                    vec2<f32>( 2.0,  0.0));
                `
          }
                return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
              }`
        }),
        entryPoint: 'main'
      },
      fragment: {
        module: t.device.createShaderModule({
          code: `
              @fragment fn main(
                @builtin(front_facing) FrontFacing : bool
                ) -> @location(0) vec4<f32> {
                var color : vec4<f32>;
                if (FrontFacing) {
                  color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
                } else {
                  color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
                }
                return color;
              }`
        }),
        entryPoint: 'main',
        targets: [{ format }]
      },
      primitive: {
        topology,
        frontFace,
        cullMode
      },
      depthStencil
    })
  );
  pass.draw(6);
  pass.end();

  t.device.queue.submit([encoder.finish()]);

  // front facing color is green, non front facing is red, background is blue
  const kCCWTriangleTopLeftColor = faceColor('ccw', frontFace, cullMode);
  const kCWTriangleBottomRightColor = faceColor('cw', frontFace, cullMode);
  t.checkCornerPixels(texture, kCCWTriangleTopLeftColor, kCWTriangleBottomRightColor);

  if (depthTexture) {
    // draw a triangle that covers all of clip space in yellow at the same depth
    // as the previous triangles with the depth test set to 'less'. We should only
    // draw yellow where the previous triangles did not.
    depthStencilAttachment.depthLoadOp = 'load';

    if (haveStencil) {
      depthStencilAttachment.stencilLoadOp = 'load';
      depthStencil.stencilFront.passOp = 'keep';
      depthStencil.stencilBack.passOp = 'keep';
    }

    const k2ndDrawColor = new Uint8Array([255, 255, 0, 255]);

    const isTopLeftCulled = faceIsCulled('ccw', frontFace, cullMode);
    const kExpectedTopLeftColor = isTopLeftCulled ? k2ndDrawColor : kCCWTriangleTopLeftColor;

    const isBottomRightCulled = faceIsCulled('cw', frontFace, cullMode);
    const kExpectedBottomRightColor = isBottomRightCulled ?
    k2ndDrawColor :
    kCWTriangleBottomRightColor;

    t.drawFullClipSpaceTriangleAndCheckCornerPixels(
      texture,
      format,
      topology,
      k2ndDrawColor,
      depthStencil,
      depthStencilAttachment,
      kExpectedTopLeftColor,
      kExpectedBottomRightColor
    );

    if (haveStencil) {
      // draw a triangle that covers all of clip space in cyan with the stencil
      // compare set to 'equal'. The reference value defaults to 0 so we should
      // only render cyan where the first two triangles did not.
      depthStencil.depthCompare = 'always';
      depthStencil.stencilFront.compare = 'equal';
      depthStencil.stencilBack.compare = 'equal';

      const k3rdDrawColor = new Uint8Array([0, 255, 255, 255]);
      const kExpectedTopLeftColor = isTopLeftCulled ? k3rdDrawColor : kCCWTriangleTopLeftColor;
      const kExpectedBottomRightColor = isBottomRightCulled ?
      k3rdDrawColor :
      kCWTriangleBottomRightColor;

      t.drawFullClipSpaceTriangleAndCheckCornerPixels(
        texture,
        format,
        topology,
        k3rdDrawColor,
        depthStencil,
        depthStencilAttachment,
        kExpectedTopLeftColor,
        kExpectedBottomRightColor
      );
    }
  }
});