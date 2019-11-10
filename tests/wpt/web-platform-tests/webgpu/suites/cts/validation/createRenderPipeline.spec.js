/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
createRenderPipeline validation tests.
`;
import { TestGroup } from '../../../framework/index.js';
import { textureFormatInfo, textureFormatParams } from '../format_info.js';
import { ValidationTest } from './validation_test.js';

class F extends ValidationTest {
  async init() {
    await Promise.all([super.init(), this.initGLSL()]);
  }

  getDescriptor(options = {}) {
    const defaultColorStates = [{
      format: 'rgba8unorm'
    }];
    const {
      primitiveTopology = 'triangle-list',
      colorStates = defaultColorStates,
      sampleCount = 1,
      depthStencilState
    } = options;
    const format = colorStates.length ? colorStates[0].format : 'rgba8unorm';
    return {
      vertexStage: this.getVertexStage(),
      fragmentStage: this.getFragmentStage(format),
      layout: this.getPipelineLayout(),
      primitiveTopology,
      colorStates,
      sampleCount,
      depthStencilState
    };
  }

  getVertexStage() {
    return {
      module: this.createShaderModule({
        code:
        /* GLSL(
         *           'vertex',
         *           `#version 450
         *             void main() {
         *               gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
         *             }
         *           `
         *         )
         */
        new Uint32Array([119734787, 66304, 524295, 21, 0, 131089, 1, 393227, 1, 1280527431, 1685353262, 808793134, 0, 196622, 0, 1, 393231, 0, 4, 1852399981, 0, 13, 196611, 2, 450, 262149, 4, 1852399981, 0, 393221, 11, 1348430951, 1700164197, 2019914866, 0, 393222, 11, 0, 1348430951, 1953067887, 7237481, 458758, 11, 1, 1348430951, 1953393007, 1702521171, 0, 458758, 11, 2, 1130327143, 1148217708, 1635021673, 6644590, 458758, 11, 3, 1130327143, 1147956341, 1635021673, 6644590, 196613, 13, 0, 327752, 11, 0, 11, 0, 327752, 11, 1, 11, 1, 327752, 11, 2, 11, 3, 327752, 11, 3, 11, 4, 196679, 11, 2, 131091, 2, 196641, 3, 2, 196630, 6, 32, 262167, 7, 6, 4, 262165, 8, 32, 0, 262187, 8, 9, 1, 262172, 10, 6, 9, 393246, 11, 7, 6, 10, 10, 262176, 12, 3, 11, 262203, 12, 13, 3, 262165, 14, 32, 1, 262187, 14, 15, 0, 262187, 6, 16, 0, 262187, 6, 17, 1065353216, 458796, 7, 18, 16, 16, 16, 17, 262176, 19, 3, 7, 327734, 2, 4, 0, 3, 131320, 5, 327745, 19, 20, 13, 15, 196670, 20, 18, 65789, 65592])
      }),
      entryPoint: 'main'
    };
  }

  getFragmentStage(format) {
    let fragColorType;

    if (format.endsWith('sint')) {
      fragColorType = 'ivec4';
    } else if (format.endsWith('uint')) {
      fragColorType = 'uvec4';
    } else {
      fragColorType = 'vec4';
    }

    const code = `
      #version 450
      layout(location = 0) out ${fragColorType} fragColor;
      void main() {
        fragColor = ${fragColorType}(0.0, 1.0, 0.0, 1.0);
      }
    `;
    return {
      module: this.makeShaderModuleFromGLSL('fragment', code),
      entryPoint: 'main'
    };
  }

  getPipelineLayout() {
    return this.device.createPipelineLayout({
      bindGroupLayouts: []
    });
  }

  createTexture(params) {
    const {
      format,
      sampleCount
    } = params;
    return this.device.createTexture({
      size: {
        width: 4,
        height: 4,
        depth: 1
      },
      usage: GPUTextureUsage.OUTPUT_ATTACHMENT,
      format,
      sampleCount
    });
  }

}

export const g = new TestGroup(F);
g.test('basic use of createRenderPipeline', t => {
  const descriptor = t.getDescriptor();
  t.device.createRenderPipeline(descriptor);
});
g.test('at least one color state is required', async t => {
  const goodDescriptor = t.getDescriptor({
    colorStates: [{
      format: 'rgba8unorm'
    }]
  }); // Control case

  t.device.createRenderPipeline(goodDescriptor); // Fail because lack of color states

  const badDescriptor = t.getDescriptor({
    colorStates: []
  });
  t.expectValidationError(() => {
    t.device.createRenderPipeline(badDescriptor);
  });
});
g.test('color formats must be renderable', async t => {
  const {
    format
  } = t.params;
  const info = textureFormatInfo[format];
  const descriptor = t.getDescriptor({
    colorStates: [{
      format
    }]
  });

  if (info.renderable && info.color) {
    // Succeeds when color format is renderable
    t.device.createRenderPipeline(descriptor);
  } else {
    // Fails because when format is non-renderable
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
}).params(textureFormatParams);
g.test('sample count must be valid', async t => {
  const {
    sampleCount,
    _success
  } = t.params;
  const descriptor = t.getDescriptor({
    sampleCount
  });

  if (_success) {
    // Succeeds when sample count is valid
    t.device.createRenderPipeline(descriptor);
  } else {
    // Fails when sample count is not 4 or 1
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
}).params([{
  sampleCount: 0,
  _success: false
}, {
  sampleCount: 1,
  _success: true
}, {
  sampleCount: 2,
  _success: false
}, {
  sampleCount: 3,
  _success: false
}, {
  sampleCount: 4,
  _success: true
}, {
  sampleCount: 8,
  _success: false
}, {
  sampleCount: 16,
  _success: false
}]);
g.test('sample count must be equal to the one of every attachment in the render pass', async t => {
  const {
    attachmentSamples,
    pipelineSamples,
    _success
  } = t.params;
  const colorTexture = t.createTexture({
    format: 'rgba8unorm',
    sampleCount: attachmentSamples
  });
  const depthStencilTexture = t.createTexture({
    format: 'depth24plus-stencil8',
    sampleCount: attachmentSamples
  });
  const renderPassDescriptorWithoutDepthStencil = {
    colorAttachments: [{
      attachment: colorTexture.createView(),
      loadValue: {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0
      }
    }]
  };
  const renderPassDescriptorWithDepthStencilOnly = {
    colorAttachments: [],
    depthStencilAttachment: {
      attachment: depthStencilTexture.createView(),
      depthLoadValue: 1.0,
      depthStoreOp: 'store',
      stencilLoadValue: 0,
      stencilStoreOp: 'store'
    }
  };
  const pipelineWithoutDepthStencil = t.device.createRenderPipeline(t.getDescriptor({
    sampleCount: pipelineSamples
  }));
  const pipelineWithDepthStencilOnly = t.device.createRenderPipeline(t.getDescriptor({
    colorStates: [],
    depthStencilState: {
      format: 'depth24plus-stencil8'
    },
    sampleCount: pipelineSamples
  }));

  for (const {
    renderPassDescriptor,
    pipeline
  } of [{
    renderPassDescriptor: renderPassDescriptorWithoutDepthStencil,
    pipeline: pipelineWithoutDepthStencil
  }, {
    renderPassDescriptor: renderPassDescriptorWithDepthStencilOnly,
    pipeline: pipelineWithDepthStencilOnly
  }]) {
    const commandEncoder = t.device.createCommandEncoder();
    const renderPass = commandEncoder.beginRenderPass(renderPassDescriptor);
    renderPass.setPipeline(pipeline);
    renderPass.endPass();
    t.expectValidationError(() => {
      commandEncoder.finish();
    }, !_success);
  }
}).params([{
  attachmentSamples: 4,
  pipelineSamples: 4,
  _success: true
}, // It is allowed to use multisampled render pass and multisampled render pipeline.
{
  attachmentSamples: 4,
  pipelineSamples: 1,
  _success: false
}, // It is not allowed to use multisampled render pass and non-multisampled render pipeline.
{
  attachmentSamples: 1,
  pipelineSamples: 4,
  _success: false
} // It is not allowed to use non-multisampled render pass and multisampled render pipeline.
]);
//# sourceMappingURL=createRenderPipeline.spec.js.map