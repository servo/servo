/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert } from '../../../../../common/util/util.js';
import { kTextureFormatInfo } from '../../../../format_info.js';
import { virtualMipSize } from '../../../../util/texture/base.js';

function makeFullscreenVertexModule(device) {
  return device.createShaderModule({
    code: `
    @vertex
    fn main(@builtin(vertex_index) VertexIndex : u32)
         -> @builtin(position) vec4<f32> {
      var pos : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0));
      return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
    }
    `,
  });
}

function getDepthTestEqualPipeline(t, format, sampleCount, expected) {
  return t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      entryPoint: 'main',
      module: makeFullscreenVertexModule(t.device),
    },
    fragment: {
      entryPoint: 'main',
      module: t.device.createShaderModule({
        code: `
        struct Outputs {
          @builtin(frag_depth) FragDepth : f32,
          @location(0) outSuccess : f32,
        };

        @fragment
        fn main() -> Outputs {
          var output : Outputs;
          output.FragDepth = f32(${expected});
          output.outSuccess = 1.0;
          return output;
        }
        `,
      }),
      targets: [{ format: 'r8unorm' }],
    },
    depthStencil: {
      format,
      depthCompare: 'equal',
      depthWriteEnabled: false,
    },
    primitive: { topology: 'triangle-list' },
    multisample: { count: sampleCount },
  });
}

function getStencilTestEqualPipeline(t, format, sampleCount) {
  return t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      entryPoint: 'main',
      module: makeFullscreenVertexModule(t.device),
    },
    fragment: {
      entryPoint: 'main',
      module: t.device.createShaderModule({
        code: `
        @fragment
        fn main() -> @location(0) f32 {
          return 1.0;
        }
        `,
      }),
      targets: [{ format: 'r8unorm' }],
    },
    depthStencil: {
      depthWriteEnabled: false,
      depthCompare: 'always',
      format,
      stencilFront: { compare: 'equal' },
      stencilBack: { compare: 'equal' },
    },
    primitive: { topology: 'triangle-list' },
    multisample: { count: sampleCount },
  });
}

const checkContents = (type, t, params, texture, state, subresourceRange) => {
  const formatInfo = kTextureFormatInfo[params.format];

  assert(params.dimension === '2d');
  for (const viewDescriptor of t.generateTextureViewDescriptorsForRendering(
    'all',
    subresourceRange
  )) {
    assert(viewDescriptor.baseMipLevel !== undefined);
    const [width, height] = virtualMipSize(
      params.dimension,
      [t.textureWidth, t.textureHeight, 1],
      viewDescriptor.baseMipLevel
    );

    const renderTexture = t.device.createTexture({
      size: [width, height, 1],
      format: 'r8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      sampleCount: params.sampleCount,
    });

    let resolveTexture = undefined;
    let resolveTarget = undefined;
    if (params.sampleCount > 1) {
      resolveTexture = t.device.createTexture({
        size: [width, height, 1],
        format: 'r8unorm',
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      });
      resolveTarget = resolveTexture.createView();
    }

    const commandEncoder = t.device.createCommandEncoder();
    commandEncoder.pushDebugGroup('checkContentsWithDepthStencil');

    const pass = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: renderTexture.createView(),
          resolveTarget,
          clearValue: [0, 0, 0, 0],
          loadOp: 'load',
          storeOp: 'store',
        },
      ],

      depthStencilAttachment: {
        view: texture.createView(viewDescriptor),
        depthStoreOp: formatInfo.depth ? 'store' : undefined,
        depthLoadOp: formatInfo.depth ? 'load' : undefined,
        stencilStoreOp: formatInfo.stencil ? 'store' : undefined,
        stencilLoadOp: formatInfo.stencil ? 'load' : undefined,
      },
    });

    switch (type) {
      case 'depth': {
        const expectedDepth = t.stateToTexelComponents[state].Depth;
        assert(expectedDepth !== undefined);

        pass.setPipeline(
          getDepthTestEqualPipeline(t, params.format, params.sampleCount, expectedDepth)
        );

        break;
      }

      case 'stencil': {
        const expectedStencil = t.stateToTexelComponents[state].Stencil;
        assert(expectedStencil !== undefined);

        pass.setPipeline(getStencilTestEqualPipeline(t, params.format, params.sampleCount));
        pass.setStencilReference(expectedStencil);
        break;
      }
    }

    pass.draw(3);
    pass.end();

    commandEncoder.popDebugGroup();
    t.queue.submit([commandEncoder.finish()]);

    t.expectSingleColor(resolveTexture || renderTexture, 'r8unorm', {
      size: [width, height, 1],
      exp: { R: 1 },
    });
  }
};

export const checkContentsByDepthTest = (...args) => checkContents('depth', ...args);

export const checkContentsByStencilTest = (...args) => checkContents('stencil', ...args);
