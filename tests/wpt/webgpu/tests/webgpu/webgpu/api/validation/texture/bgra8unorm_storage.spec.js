/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests for capabilities added by bgra8unorm-storage flag.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert } from '../../../../common/util/util.js';
import { kTextureUsages } from '../../../capability_info.js';
import { GPUConst } from '../../../constants.js';
import { kAllCanvasTypes, createCanvas } from '../../../util/create_elements.js';
import { ValidationTest } from '../validation_test.js';

class BGRA8UnormStorageValidationTests extends ValidationTest {
  testCreateShaderModuleWithBGRA8UnormStorage(shaderType, success) {
    let shaderDeclaration = '';
    switch (shaderType) {
      case 'fragment':
        shaderDeclaration = '@fragment';
        break;
      case 'compute':
        shaderDeclaration = '@compute @workgroup_size(1)';
        break;
    }

    this.expectValidationError(() => {
      this.device.createShaderModule({
        code: `
      @group(0) @binding(1) var outputTex: texture_storage_2d<bgra8unorm, write>;
      ${shaderDeclaration} fn main() {
        textureStore(outputTex, vec2<i32>(), vec4<f32>());
      }
      `,
      });
    }, !success);
  }
}

export const g = makeTestGroup(BGRA8UnormStorageValidationTests);

g.test('create_texture')
  .desc(
    `
Test that it is valid to create bgra8unorm texture with STORAGE usage iff the feature
bgra8unorm-storage is enabled. Note, the createTexture test suite covers the validation cases where
this feature is not enabled, which are skipped here.
`
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('bgra8unorm-storage');
  })
  .fn(t => {
    const descriptor = {
      size: [1, 1, 1],
      format: 'bgra8unorm',
      usage: GPUConst.TextureUsage.STORAGE,
    };
    t.device.createTexture(descriptor);
  });

g.test('create_bind_group_layout')
  .desc(
    `
Test that it is valid to create GPUBindGroupLayout that uses bgra8unorm as storage texture format
iff the feature bgra8unorm-storage is enabled. Note, the createBindGroupLayout test suite covers the
validation cases where this feature is not enabled, which are skipped here.
`
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('bgra8unorm-storage');
  })
  .fn(t => {
    t.device.createBindGroupLayout({
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.COMPUTE,
          storageTexture: { format: 'bgra8unorm' },
        },
      ],
    });
  });

g.test('create_shader_module_with_bgra8unorm_storage')
  .desc(
    `
Test that it is valid to declare the format of a storage texture as bgra8unorm in a shader module if
the feature bgra8unorm-storage is enabled.
`
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('bgra8unorm-storage');
  })
  .params(u => u.combine('shaderType', ['fragment', 'compute']))
  .fn(t => {
    const { shaderType } = t.params;

    t.testCreateShaderModuleWithBGRA8UnormStorage(shaderType, true);
  });

g.test('create_shader_module_without_bgra8unorm_storage')
  .desc(
    `
Test that it is invalid to declare the format of a storage texture as bgra8unorm in a shader module
if the feature bgra8unorm-storage is not enabled.
`
  )
  .params(u => u.combine('shaderType', ['fragment', 'compute']))
  .fn(t => {
    const { shaderType } = t.params;

    t.testCreateShaderModuleWithBGRA8UnormStorage(shaderType, false);
  });

g.test('configure_storage_usage_on_canvas_context_without_bgra8unorm_storage')
  .desc(
    `
Test that it is invalid to configure a GPUCanvasContext to 'GPUStorageBinding' usage with
'bgra8unorm' format on a GPUDevice with 'bgra8unorm-storage' disabled.
`
  )
  .params(u =>
    u
      .combine('canvasType', kAllCanvasTypes)
      .beginSubcases()
      .expand('usage', p => {
        const usageSet = new Set();
        for (const usage0 of kTextureUsages) {
          for (const usage1 of kTextureUsages) {
            usageSet.add(usage0 | usage1);
          }
        }
        return usageSet;
      })
  )
  .fn(t => {
    const { canvasType, usage } = t.params;
    const canvas = createCanvas(t, canvasType, 1, 1);
    const ctx = canvas.getContext('webgpu');
    assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

    const requiredStorageBinding = !!(usage & GPUTextureUsage.STORAGE_BINDING);
    t.expectValidationError(() => {
      ctx.configure({
        device: t.device,
        format: 'bgra8unorm',
        usage,
      });
    }, requiredStorageBinding);
  });

g.test('configure_storage_usage_on_canvas_context_with_bgra8unorm_storage')
  .desc(
    `
Test that it is valid to configure a GPUCanvasContext with GPUStorageBinding usage and a GPUDevice
with 'bgra8unorm-storage' enabled.
`
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase('bgra8unorm-storage');
  })
  .params(u =>
    u
      .combine('canvasType', kAllCanvasTypes)
      .beginSubcases()
      .expand('usage', p => {
        const usageSet = new Set();
        for (const usage of kTextureUsages) {
          usageSet.add(usage | GPUConst.TextureUsage.STORAGE_BINDING);
        }
        return usageSet;
      })
  )
  .fn(t => {
    const { canvasType, usage } = t.params;
    const canvas = createCanvas(t, canvasType, 1, 1);
    const ctx = canvas.getContext('webgpu');
    assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

    ctx.configure({
      device: t.device,
      format: 'bgra8unorm',
      usage,
    });

    const currentTexture = ctx.getCurrentTexture();
    const bindGroupLayout = t.device.createBindGroupLayout({
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.FRAGMENT,
          storageTexture: { access: 'write-only', format: currentTexture.format },
        },
      ],
    });
    t.device.createBindGroup({
      layout: bindGroupLayout,
      entries: [
        {
          binding: 0,
          resource: currentTexture.createView(),
        },
      ],
    });
  });
