/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
setBindGroup validation tests.
`;
import { pcombine, poptions } from '../../../common/framework/params.js';
import { TestGroup } from '../../../common/framework/test_group.js';
import { ValidationTest } from './validation_test.js';

class F extends ValidationTest {
  makeAttachmentTexture() {
    return this.device.createTexture({
      format: 'rgba8unorm',
      size: {
        width: 16,
        height: 16,
        depth: 1
      },
      usage: GPUTextureUsage.OUTPUT_ATTACHMENT
    });
  }

  testComputePass(bindGroup, dynamicOffsets) {
    const encoder = this.device.createCommandEncoder();
    const computePass = encoder.beginComputePass();
    computePass.setBindGroup(0, bindGroup, dynamicOffsets);
    computePass.endPass();
    encoder.finish();
  }

  testRenderPass(bindGroup, dynamicOffsets) {
    const encoder = this.device.createCommandEncoder();
    const renderPass = encoder.beginRenderPass({
      colorAttachments: [{
        attachment: this.makeAttachmentTexture().createView(),
        loadValue: {
          r: 1.0,
          g: 0.0,
          b: 0.0,
          a: 1.0
        }
      }]
    });
    renderPass.setBindGroup(0, bindGroup, dynamicOffsets);
    renderPass.endPass();
    encoder.finish();
  }

  testRenderBundle(bindGroup, dynamicOffsets) {
    const encoder = this.device.createRenderBundleEncoder({
      colorFormats: ['rgba8unorm']
    });
    encoder.setBindGroup(0, bindGroup, dynamicOffsets);
    encoder.finish();
  }

}

export const g = new TestGroup(F);
g.test('dynamic offsets passed but not expected/compute pass', async t => {
  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: []
  });
  const bindGroup = t.device.createBindGroup({
    layout: bindGroupLayout,
    entries: []
  });
  const {
    type
  } = t.params;
  const dynamicOffsets = [0];
  t.expectValidationError(() => {
    if (type === 'compute') {
      const encoder = t.device.createCommandEncoder();
      const computePass = encoder.beginComputePass();
      computePass.setBindGroup(0, bindGroup, dynamicOffsets);
      computePass.endPass();
      encoder.finish();
    } else if (type === 'renderpass') {
      const encoder = t.device.createCommandEncoder();
      const renderPass = encoder.beginRenderPass({
        colorAttachments: [{
          attachment: t.makeAttachmentTexture().createView(),
          loadValue: {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0
          }
        }]
      });
      renderPass.setBindGroup(0, bindGroup, dynamicOffsets);
      renderPass.endPass();
      encoder.finish();
    } else if (type === 'renderbundle') {
      const encoder = t.device.createRenderBundleEncoder({
        colorFormats: ['rgba8unorm']
      });
      encoder.setBindGroup(0, bindGroup, dynamicOffsets);
      encoder.finish();
    } else {
      t.fail();
    }
  });
}).params(poptions('type', ['compute', 'renderpass', 'renderbundle']));
g.test('dynamic offsets match expectations in pass encoder', async t => {
  // Dynamic buffer offsets require offset to be divisible by 256
  const MIN_DYNAMIC_BUFFER_OFFSET_ALIGNMENT = 256;
  const BINDING_SIZE = 9;
  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT,
      type: 'uniform-buffer',
      hasDynamicOffset: true
    }, {
      binding: 1,
      visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT,
      type: 'storage-buffer',
      hasDynamicOffset: true
    }]
  });
  const uniformBuffer = t.device.createBuffer({
    size: 2 * MIN_DYNAMIC_BUFFER_OFFSET_ALIGNMENT + 8,
    usage: GPUBufferUsage.UNIFORM
  });
  const storageBuffer = t.device.createBuffer({
    size: 2 * MIN_DYNAMIC_BUFFER_OFFSET_ALIGNMENT + 8,
    usage: GPUBufferUsage.STORAGE
  });
  const bindGroup = t.device.createBindGroup({
    layout: bindGroupLayout,
    entries: [{
      binding: 0,
      resource: {
        buffer: uniformBuffer,
        size: BINDING_SIZE
      }
    }, {
      binding: 1,
      resource: {
        buffer: storageBuffer,
        size: BINDING_SIZE
      }
    }]
  });
  const {
    type,
    dynamicOffsets,
    _success
  } = t.params;
  t.expectValidationError(() => {
    if (type === 'compute') {
      t.testComputePass(bindGroup, dynamicOffsets);
    } else if (type === 'renderpass') {
      t.testRenderPass(bindGroup, dynamicOffsets);
    } else if (type === 'renderbundle') {
      t.testRenderBundle(bindGroup, dynamicOffsets);
    } else {
      t.fail();
    }

    t.testComputePass(bindGroup, dynamicOffsets);
  }, !_success);
}).params(pcombine(poptions('type', ['compute', 'renderpass', 'renderbundle']), [{
  dynamicOffsets: [256, 0],
  _success: true
}, // Dynamic offsets aligned
{
  dynamicOffsets: [1, 2],
  _success: false
}, // Dynamic offsets not aligned
// Wrong number of dynamic offsets
{
  dynamicOffsets: [256, 0, 0],
  _success: false
}, {
  dynamicOffsets: [256],
  _success: false
}, {
  dynamicOffsets: [],
  _success: false
}, // Dynamic uniform buffer out of bounds because of binding size
{
  dynamicOffsets: [512, 0],
  _success: false
}, {
  dynamicOffsets: [1024, 0],
  _success: false
}, {
  dynamicOffsets: [Number.MAX_SAFE_INTEGER, 0],
  _success: false
}, // Dynamic storage buffer out of bounds because of binding size
{
  dynamicOffsets: [0, 512],
  _success: false
}, {
  dynamicOffsets: [0, 1024],
  _success: false
}, {
  dynamicOffsets: [0, Number.MAX_SAFE_INTEGER],
  _success: false
}])); // TODO: test error bind group
//# sourceMappingURL=setBindGroup.spec.js.map