/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for device lost induced via destroy.
  - Tests that prior to device destruction, valid APIs do not generate errors (control case).
  - After device destruction, runs the same APIs. No expected observable results, so test crash or future failures are the only current failure indicators.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { assert } from '../../../../../common/util/util.js';
import {
  allBindingEntries,
  bindingTypeInfo,
  kBindableResources,
  kBufferUsageKeys,
  kBufferUsageInfo,
  kBufferUsageCopy,
  kBufferUsageCopyInfo,
  kQueryTypes,
  kTextureUsageType,
  kTextureUsageTypeInfo,
  kTextureUsageCopy,
  kTextureUsageCopyInfo,
  kShaderStageKeys } from
'../../../../capability_info.js';
import {
  kCompressedTextureFormats,
  kRegularTextureFormats,
  kRenderableColorTextureFormats,
  kTextureFormatInfo } from
'../../../../format_info.js';

import {
  createCanvas,
  kAllCanvasTypes,
  kValidCanvasContextIds } from
'../../../../util/create_elements.js';
import {
  startPlayingAndWaitForVideo,
  getVideoElement,
  getVideoFrameFromVideoElement } from
'../../../../web_platform/util.js';
import { ValidationTest } from '../../validation_test.js';

const kCommandValidationStages = ['finish', 'submit'];


class DeviceDestroyTests extends ValidationTest {
  /**
   * Expects that `fn` does not produce any errors before the device is destroyed, and then calls
   * `fn` after the device is destroyed without any specific expectation. If `awaitLost` is true, we
   * also wait for device.lost to resolve before executing `fn` in the destroy case.
   */
  async executeAfterDestroy(fn, awaitLost) {
    this.expectDeviceLost('destroyed');

    this.expectValidationError(fn, false);
    this.device.destroy();
    if (awaitLost) {
      const lostInfo = await this.device.lost;
      this.expect(lostInfo.reason === 'destroyed');
    }
    fn();
  }

  /**
   * Expects that encoders can finish and submit the resulting commands before the device is
   * destroyed, then repeats the same process after the device is destroyed without any specific
   * expectations.
   * There are two valid stages: 'finish' and 'submit'.
   *   'finish': Tests [encode, finish] and [encoder, destroy, finish]
   *   'submit': Tests [encoder, finish, submit] and [encoder, finish, destroy, submit]
   */
  async executeCommandsAfterDestroy(
  stage,
  awaitLost,
  encoderType,
  fn)
  {
    this.expectDeviceLost('destroyed');

    switch (stage) {
      case 'finish':{
          // Control case
          fn(this.createEncoder(encoderType)).validateFinish(true);
          // Validation case
          const encoder = fn(this.createEncoder(encoderType));
          await this.executeAfterDestroy(() => {
            encoder.finish();
          }, awaitLost);
          break;
        }
      case 'submit':{
          // Control case
          fn(this.createEncoder(encoderType)).validateFinishAndSubmit(true, true);
          // Validation case
          const commands = fn(this.createEncoder(encoderType)).validateFinish(true);
          await this.executeAfterDestroy(() => {
            this.queue.submit([commands]);
          }, awaitLost);
          break;
        }
    }
  }
}

export const g = makeTestGroup(DeviceDestroyTests);

g.test('createBuffer').
desc(
  `
Tests creating buffers on destroyed device. Tests valid combinations of:
  - Various usages
  - Mapped at creation or not
  `
).
params((u) =>
u.
combine('usageType', kBufferUsageKeys).

combine('usageCopy', kBufferUsageCopy).
combine('awaitLost', [true, false]).
filter(({ usageType, usageCopy }) => {
  if (usageType === 'COPY_SRC' || usageType === 'COPY_DST') {
    return false;
  }
  if (usageType === 'MAP_READ') {
    return usageCopy === 'COPY_NONE' || usageCopy === 'COPY_DST';
  }
  if (usageType === 'MAP_WRITE') {
    return usageCopy === 'COPY_NONE' || usageCopy === 'COPY_SRC';
  }
  return true;
}).
combine('mappedAtCreation', [true, false])
).
fn(async (t) => {
  const { awaitLost, usageType, usageCopy, mappedAtCreation } = t.params;
  await t.executeAfterDestroy(() => {
    t.device.createBuffer({
      size: 16,
      usage: kBufferUsageInfo[usageType] | kBufferUsageCopyInfo[usageCopy],
      mappedAtCreation
    });
  }, awaitLost);
});

g.test('createTexture,2d,uncompressed_format').
desc(
  `
Tests creating 2d uncompressed textures on destroyed device. Tests valid combinations of:
  - Various uncompressed texture formats
  - Various usages
  `
).
params((u) =>
u.
combine('format', kRegularTextureFormats).

combine('usageType', kTextureUsageType).
combine('usageCopy', kTextureUsageCopy).
combine('awaitLost', [true, false]).
filter(({ format, usageType }) => {
  const info = kTextureFormatInfo[format];
  return !(
  !info.colorRender && usageType === 'render' ||
  !info.color.storage && usageType === 'storage');

})
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
}).
fn(async (t) => {
  const { awaitLost, format, usageType, usageCopy } = t.params;
  const { blockWidth, blockHeight } = kTextureFormatInfo[format];
  await t.executeAfterDestroy(() => {
    t.device.createTexture({
      size: { width: blockWidth, height: blockHeight },
      usage: kTextureUsageTypeInfo[usageType] | kTextureUsageCopyInfo[usageCopy],
      format
    });
  }, awaitLost);
});

g.test('createTexture,2d,compressed_format').
desc(
  `
Tests creating 2d compressed textures on destroyed device. Tests valid combinations of:
  - Various compressed texture formats
  - Various usages
  `
).
params((u) =>
u.
combine('format', kCompressedTextureFormats).

combine('usageType', kTextureUsageType).
combine('usageCopy', kTextureUsageCopy).
combine('awaitLost', [true, false]).
filter(({ format, usageType }) => {
  const info = kTextureFormatInfo[format];
  return !(
  !info.colorRender && usageType === 'render' ||
  !info.color.storage && usageType === 'storage');

})
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  t.selectDeviceOrSkipTestCase(kTextureFormatInfo[format].feature);
}).
fn(async (t) => {
  const { awaitLost, format, usageType, usageCopy } = t.params;
  const { blockWidth, blockHeight } = kTextureFormatInfo[format];
  await t.executeAfterDestroy(() => {
    t.device.createTexture({
      size: { width: blockWidth, height: blockHeight },
      usage: kTextureUsageTypeInfo[usageType] | kTextureUsageCopyInfo[usageCopy],
      format
    });
  }, awaitLost);
});

g.test('createView,2d,uncompressed_format').
desc(
  `
Tests creating texture views on 2d uncompressed textures from destroyed device. Tests valid combinations of:
  - Various uncompressed texture formats
  - Various usages
  `
).
params((u) =>
u.
combine('format', kRegularTextureFormats).

combine('usageType', kTextureUsageType).
combine('usageCopy', kTextureUsageCopy).
combine('awaitLost', [true, false]).
filter(({ format, usageType }) => {
  const info = kTextureFormatInfo[format];
  return !(
  !info.colorRender && usageType === 'render' ||
  !info.color.storage && usageType === 'storage');

})
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
}).
fn(async (t) => {
  const { awaitLost, format, usageType, usageCopy } = t.params;
  const { blockWidth, blockHeight } = kTextureFormatInfo[format];
  const texture = t.device.createTexture({
    size: { width: blockWidth, height: blockHeight },
    usage: kTextureUsageTypeInfo[usageType] | kTextureUsageCopyInfo[usageCopy],
    format
  });
  await t.executeAfterDestroy(() => {
    texture.createView({ format });
  }, awaitLost);
});

g.test('createView,2d,compressed_format').
desc(
  `
Tests creating texture views on 2d compressed textures from destroyed device. Tests valid combinations of:
  - Various compressed texture formats
  - Various usages
  `
).
params((u) =>
u.
combine('format', kCompressedTextureFormats).

combine('usageType', kTextureUsageType).
combine('usageCopy', kTextureUsageCopy).
combine('awaitLost', [true, false]).
filter(({ format, usageType }) => {
  const info = kTextureFormatInfo[format];
  return !(
  !info.colorRender && usageType === 'render' ||
  !info.color.storage && usageType === 'storage');

})
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  t.selectDeviceOrSkipTestCase(kTextureFormatInfo[format].feature);
}).
fn(async (t) => {
  const { awaitLost, format, usageType, usageCopy } = t.params;
  const { blockWidth, blockHeight } = kTextureFormatInfo[format];
  const texture = t.device.createTexture({
    size: { width: blockWidth, height: blockHeight },
    usage: kTextureUsageTypeInfo[usageType] | kTextureUsageCopyInfo[usageCopy],
    format
  });
  await t.executeAfterDestroy(() => {
    texture.createView({ format });
  }, awaitLost);
});

g.test('createSampler').
desc(
  `
Tests creating samplers on destroyed device.
  `
).
params((u) => u.combine('awaitLost', [true, false])).
fn(async (t) => {
  const { awaitLost } = t.params;
  await t.executeAfterDestroy(() => {
    t.device.createSampler();
  }, awaitLost);
});

g.test('createBindGroupLayout').
desc(
  `
Tests creating bind group layouts on destroyed device. Tests valid combinations of:
  - Various valid binding entries
  - Maximum set of visibility for each binding entry
  `
).
params((u) => u.combine('entry', allBindingEntries(false)).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { awaitLost, entry } = t.params;
  const visibility = bindingTypeInfo(entry).validStages;
  await t.executeAfterDestroy(() => {
    t.device.createBindGroupLayout({
      entries: [{ binding: 0, visibility, ...entry }]
    });
  }, awaitLost);
});

g.test('createBindGroup').
desc(
  `
Tests creating bind group on destroyed device. Tests valid combinations of:
  - Various bound resource types
  - Various valid binding entries
  - Maximum set of visibility for each binding entry
  `
).
desc(`A destroyed device should not be able to create any valid bind groups.`).
params((u) =>
u.
combine('resourceType', kBindableResources).
combine('entry', allBindingEntries(false)).
filter(({ resourceType, entry }) => {
  const info = bindingTypeInfo(entry);
  switch (info.resource) {
    // Either type of sampler may be bound to a filtering sampler binding.
    case 'filtSamp':
      return resourceType === 'filtSamp' || resourceType === 'nonFiltSamp';
    // But only non-filtering samplers can be used with non-filtering sampler bindings.
    case 'nonFiltSamp':
      return resourceType === 'nonFiltSamp';
    default:
      return info.resource === resourceType;
  }
}).

combine('awaitLost', [true, false])
).
fn(async (t) => {
  const { awaitLost, resourceType, entry } = t.params;
  const visibility = bindingTypeInfo(entry).validStages;
  const layout = t.device.createBindGroupLayout({
    entries: [{ binding: 0, visibility, ...entry }]
  });
  const resource = t.getBindingResource(resourceType);
  await t.executeAfterDestroy(() => {
    t.device.createBindGroup({ layout, entries: [{ binding: 0, resource }] });
  }, awaitLost);
});

g.test('createPipelineLayout').
desc(
  `
Tests creating pipeline layouts on destroyed device. Tests valid combinations of:
  - Various bind groups with valid binding entries
  - Maximum set of visibility for each binding entry
  `
).
params((u) => u.combine('entry', allBindingEntries(false)).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { awaitLost, entry } = t.params;
  const visibility = bindingTypeInfo(entry).validStages;
  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [{ binding: 0, visibility, ...entry }]
  });
  await t.executeAfterDestroy(() => {
    t.device.createPipelineLayout({
      bindGroupLayouts: [bindGroupLayout]
    });
  }, awaitLost);
});

g.test('createShaderModule').
desc(
  `
Tests creating shader modules on destroyed device.
  - Tests all shader stages: vertex, fragment, compute
  `
).
params((u) => u.combine('stage', kShaderStageKeys).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { awaitLost, stage } = t.params;
  await t.executeAfterDestroy(() => {
    t.device.createShaderModule({ code: t.getNoOpShaderCode(stage) });
  }, awaitLost);
});

g.test('createComputePipeline').
desc(
  `
Tests creating compute pipeline on destroyed device.
  - Tests with a valid no-op compute shader
  `
).
params((u) => u.combine('awaitLost', [true, false])).
fn(async (t) => {
  const { awaitLost } = t.params;
  const cShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('COMPUTE') });
  await t.executeAfterDestroy(() => {
    t.device.createComputePipeline({
      layout: 'auto',
      compute: { module: cShader, entryPoint: 'main' }
    });
  }, awaitLost);
});

g.test('createRenderPipeline').
desc(
  `
Tests creating render pipeline on destroyed device.
  - Tests with valid no-op vertex and fragment shaders
  `
).
params((u) => u.combine('awaitLost', [true, false])).
fn(async (t) => {
  const { awaitLost } = t.params;
  const vShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('VERTEX') });
  const fShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('FRAGMENT') });
  await t.executeAfterDestroy(() => {
    t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module: vShader, entryPoint: 'main' },
      fragment: {
        module: fShader,
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm', writeMask: 0 }]
      }
    });
  }, awaitLost);
});

g.test('createComputePipelineAsync').
desc(
  `
Tests creating a pipeline asynchronously while destroying the device and on a destroyed device
- valid={true, false}, use an invalid or valid pipeline descriptor
- awaitLost={true, false}, check results before/after waiting for the device lost promise
  `
).
params((u) => u.combine('valid', [true, false]).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { valid, awaitLost } = t.params;
  const cShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('COMPUTE') });
  const fn = () =>
  t.device.createComputePipelineAsync({
    layout: 'auto',
    compute: { module: cShader, entryPoint: valid ? 'main' : 'does_not_exist' }
  });

  // Kick off async creation
  const p = fn();

  // Track whether or not the device is lost.
  let isLost = false;
  void t.device.lost.then(() => {
    isLost = true;
  });

  if (valid) {
    // The async creation should resolve successfully.
    t.shouldResolve(
      (async () => {
        const pipeline = await p;
        assert(pipeline instanceof GPUComputePipeline, 'Pipeline was not a GPUComputePipeline');
      })()
    );
  } else {
    // The async creation should resolve successfully if the device is lost.
    // If the device is not lost, it should see a validation error.
    // Note: this could be a race!
    t.shouldResolve(
      p.then(
        (pipeline) => {
          assert(
            isLost,
            'Invalid async creation should "succeed" if the device is already lost.'
          );
          assert(pipeline instanceof GPUComputePipeline, 'Pipeline was not a GPUComputePipeline');
        },
        (err) => {
          assert(
            !isLost,
            'Invalid async creation should only fail if the device is not yet lost.'
          );
          assert(err instanceof GPUPipelineError, 'Error was not a GPUPipelineError');
          assert(err.reason === 'validation', 'Expected validation error');
        }
      )
    );
  }

  // Destroy the device, and expect it to be lost.
  t.expectDeviceLost('destroyed');
  t.device.destroy();
  if (awaitLost) {
    const lostInfo = await t.device.lost;
    t.expect(lostInfo.reason === 'destroyed');
  }

  // After device destroy, creation should still resolve successfully.
  t.shouldResolve(
    (async () => {
      const pipeline = await fn();
      assert(pipeline instanceof GPUComputePipeline, 'Pipeline was not a GPUComputePipeline');
    })()
  );
});

g.test('createRenderPipelineAsync').
desc(
  `
Tests creating a pipeline asynchronously while destroying the device and on a destroyed device
- valid={true, false}, use an invalid or valid pipeline descriptor
- awaitLost={true, false}, check results before/after waiting for the device lost promise
  `
).
params((u) => u.combine('valid', [true, false]).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { valid, awaitLost } = t.params;
  const vShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('VERTEX') });
  const fShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('FRAGMENT') });
  const fn = () =>
  t.device.createRenderPipelineAsync({
    layout: 'auto',
    vertex: { module: vShader, entryPoint: 'main' },
    fragment: {
      module: fShader,
      entryPoint: valid ? 'main' : 'does_not_exist',
      targets: [{ format: 'rgba8unorm', writeMask: 0 }]
    }
  });

  // Kick off async creation
  const p = fn();

  // Track whether or not the device is lost.
  let isLost = false;
  void t.device.lost.then(() => {
    isLost = true;
  });

  if (valid) {
    // The async creation should resolve successfully.
    t.shouldResolve(
      (async () => {
        const pipeline = await p;
        assert(pipeline instanceof GPURenderPipeline, 'Pipeline was not a GPURenderPipeline');
      })()
    );
  } else {
    // The async creation should resolve successfully if the device is lost.
    // If the device is not lost, it should see a validation error.
    // Note: this could be a race!
    t.shouldResolve(
      p.then(
        (pipeline) => {
          assert(
            isLost,
            'Invalid async creation should "succeed" if the device is already lost.'
          );
          assert(pipeline instanceof GPURenderPipeline, 'Pipeline was not a GPURenderPipeline');
        },
        (err) => {
          assert(
            !isLost,
            'Invalid async creation should only fail if the device is not yet lost.'
          );
          assert(err instanceof GPUPipelineError, 'Error was not a GPUPipelineError');
          assert(err.reason === 'validation', 'Expected validation error');
        }
      )
    );
  }

  // Destroy the device, and expect it to be lost.
  t.expectDeviceLost('destroyed');
  t.device.destroy();
  if (awaitLost) {
    const lostInfo = await t.device.lost;
    t.expect(lostInfo.reason === 'destroyed');
  }

  // After device destroy, creation should still resolve successfully.
  t.shouldResolve(
    (async () => {
      const pipeline = await fn();
      assert(pipeline instanceof GPURenderPipeline, 'Pipeline was not a GPURenderPipeline');
    })()
  );
});

g.test('createCommandEncoder').
desc(
  `
Tests creating command encoders on destroyed device.
  `
).
params((u) => u.combine('awaitLost', [true, false])).
fn(async (t) => {
  const { awaitLost } = t.params;
  await t.executeAfterDestroy(() => {
    t.device.createCommandEncoder();
  }, awaitLost);
});

g.test('createRenderBundleEncoder').
desc(
  `
Tests creating render bundle encoders on destroyed device.
  - Tests various renderable texture color formats
  `
).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).

combine('awaitLost', [true, false])
).
fn(async (t) => {
  const { awaitLost, format } = t.params;
  await t.executeAfterDestroy(() => {
    t.device.createRenderBundleEncoder({ colorFormats: [format] });
  }, awaitLost);
});

g.test('createQuerySet').
desc(
  `
Tests creating query sets on destroyed device.
  - Tests various query set types
  `
).
params((u) => u.combine('type', kQueryTypes).combine('awaitLost', [true, false])).
beforeAllSubcases((t) => {
  const { type } = t.params;
  t.selectDeviceForQueryTypeOrSkipTestCase(type);
}).
fn(async (t) => {
  const { awaitLost, type } = t.params;
  await t.executeAfterDestroy(() => {
    t.device.createQuerySet({ type, count: 4 });
  }, awaitLost);
});

g.test('importExternalTexture').
desc(
  `
Tests import external texture on destroyed device. Tests valid combinations of:
  - Various valid source type
  `
).
params((u) =>
u.
combine('sourceType', ['VideoElement', 'VideoFrame']).

combine('awaitLost', [true, false])
).
fn(async (t) => {
  const { awaitLost, sourceType } = t.params;

  const videoElement = getVideoElement(t, 'four-colors-vp9-bt601.webm');
  if (!('requestVideoFrameCallback' in videoElement)) {
    t.skip('HTMLVideoElement.requestVideoFrameCallback is not supported');
  }

  let source;
  await startPlayingAndWaitForVideo(videoElement, async () => {
    source =
    sourceType === 'VideoFrame' ?
    await getVideoFrameFromVideoElement(t, videoElement) :
    videoElement;

    await t.executeAfterDestroy(() => {
      t.device.createBindGroup({
        layout: t.device.createBindGroupLayout({
          entries: [{ binding: 0, visibility: GPUShaderStage.FRAGMENT, externalTexture: {} }]
        }),
        entries: [
        {
          binding: 0,
          resource: t.device.importExternalTexture({ source })
        }]

      });
    }, awaitLost);
  });
});

g.test('command,copyBufferToBuffer').
desc(
  `
Tests copyBufferToBuffer command with various uncompressed formats on destroyed device.
  `
).
params((u) => u.combine('stage', kCommandValidationStages).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { stage, awaitLost } = t.params;
  const kBufferSize = 16;
  const src = t.device.createBuffer({
    size: kBufferSize,
    usage: GPUBufferUsage.COPY_SRC
  });
  const dst = t.device.createBuffer({
    size: kBufferSize,
    usage: GPUBufferUsage.COPY_DST
  });
  await t.executeCommandsAfterDestroy(stage, awaitLost, 'non-pass', (maker) => {
    maker.encoder.copyBufferToBuffer(src, 0, dst, 0, kBufferSize);
    return maker;
  });
});

g.test('command,copyBufferToTexture').
desc(
  `
Tests copyBufferToTexture command on destroyed device.
  - Tests finishing encoding on destroyed device
  - Tests submitting command on destroyed device
  `
).
params((u) => u.combine('stage', kCommandValidationStages).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { stage, awaitLost } = t.params;
  const format = 'rgba32uint';
  const {
    color: { bytes: bytesPerBlock },
    blockWidth,
    blockHeight
  } = kTextureFormatInfo[format];
  const src = {
    buffer: t.device.createBuffer({
      size: bytesPerBlock,
      usage: GPUBufferUsage.COPY_SRC
    })
  };
  const dst = {
    texture: t.device.createTexture({
      size: { width: blockWidth, height: blockHeight },
      usage: GPUTextureUsage.COPY_DST,
      format
    })
  };
  const copySize = { width: blockWidth, height: blockHeight };
  await t.executeCommandsAfterDestroy(stage, awaitLost, 'non-pass', (maker) => {
    maker.encoder.copyBufferToTexture(src, dst, copySize);
    return maker;
  });
});

g.test('command,copyTextureToBuffer').
desc(
  `
Tests copyTextureToBuffer command on destroyed device.
  - Tests finishing encoding on destroyed device
  - Tests submitting command on destroyed device
  `
).
params((u) => u.combine('stage', kCommandValidationStages).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { stage, awaitLost } = t.params;
  const format = 'rgba32uint';
  const {
    color: { bytes: bytesPerBlock },
    blockWidth,
    blockHeight
  } = kTextureFormatInfo[format];
  const src = {
    texture: t.device.createTexture({
      size: { width: blockWidth, height: blockHeight },
      usage: GPUTextureUsage.COPY_SRC,
      format
    })
  };
  const dst = {
    buffer: t.device.createBuffer({
      size: bytesPerBlock,
      usage: GPUBufferUsage.COPY_DST
    })
  };
  const copySize = { width: blockWidth, height: blockHeight };
  await t.executeCommandsAfterDestroy(stage, awaitLost, 'non-pass', (maker) => {
    maker.encoder.copyTextureToBuffer(src, dst, copySize);
    return maker;
  });
});

g.test('command,copyTextureToTexture').
desc(
  `
Tests copyTextureToTexture command on destroyed device.
  - Tests finishing encoding on destroyed device
  - Tests submitting command on destroyed device
  `
).
params((u) => u.combine('stage', kCommandValidationStages).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { stage, awaitLost } = t.params;
  const format = 'rgba32uint';
  const { blockWidth, blockHeight } = kTextureFormatInfo[format];
  const src = {
    texture: t.device.createTexture({
      size: { width: blockWidth, height: blockHeight },
      usage: GPUTextureUsage.COPY_SRC,
      format
    })
  };
  const dst = {
    texture: t.device.createTexture({
      size: { width: blockWidth, height: blockHeight },
      usage: GPUBufferUsage.COPY_DST,
      format
    })
  };
  const copySize = { width: blockWidth, height: blockHeight };
  await t.executeCommandsAfterDestroy(stage, awaitLost, 'non-pass', (maker) => {
    maker.encoder.copyTextureToTexture(src, dst, copySize);
    return maker;
  });
});

g.test('command,clearBuffer').
desc(
  `
Tests encoding and finishing a clearBuffer command on destroyed device.
  - Tests finishing encoding on destroyed device
  - Tests submitting command on destroyed device
  `
).
params((u) => u.combine('stage', kCommandValidationStages).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { stage, awaitLost } = t.params;
  const kBufferSize = 16;
  const buffer = t.device.createBuffer({
    size: kBufferSize,
    usage: GPUBufferUsage.COPY_SRC
  });
  await t.executeCommandsAfterDestroy(stage, awaitLost, 'non-pass', (maker) => {
    maker.encoder.clearBuffer(buffer, 0, kBufferSize);
    return maker;
  });
});

g.test('command,writeTimestamp').
desc(
  `
Tests encoding and finishing a writeTimestamp command on destroyed device.
  - Tests finishing encoding on destroyed device
  - Tests submitting command on destroyed device
  `
).
params((u) =>
u.
combine('type', kQueryTypes).

combine('stage', kCommandValidationStages).
combine('awaitLost', [true, false])
).
beforeAllSubcases((t) => {
  const { type } = t.params;

  // writeTimestamp is only available for devices that enable the 'timestamp-query' feature.
  const queryTypes = ['timestamp'];
  if (type !== 'timestamp') {
    queryTypes.push(type);
  }

  t.selectDeviceForQueryTypeOrSkipTestCase(queryTypes);
}).
fn(async (t) => {
  const { type, stage, awaitLost } = t.params;
  const querySet = t.device.createQuerySet({ type, count: 2 });
  await t.executeCommandsAfterDestroy(stage, awaitLost, 'non-pass', (maker) => {
    try {

      maker.encoder.writeTimestamp(querySet, 0);
    } catch (ex) {
      t.skipIf(ex instanceof TypeError, 'writeTimestamp is actually not available');
    }
    return maker;
  });
});

g.test('command,resolveQuerySet').
desc(
  `
Tests encoding and finishing a resolveQuerySet command on destroyed device.
  - Tests finishing encoding on destroyed device
  - Tests submitting command on destroyed device
  `
).
params((u) => u.combine('stage', kCommandValidationStages).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { stage, awaitLost } = t.params;
  const kQueryCount = 2;
  const querySet = t.createQuerySetWithState('valid');
  const destination = t.createBufferWithState('valid', {
    size: kQueryCount * 8,
    usage: GPUBufferUsage.QUERY_RESOLVE
  });
  await t.executeCommandsAfterDestroy(stage, awaitLost, 'non-pass', (maker) => {
    maker.encoder.resolveQuerySet(querySet, 0, 1, destination, 0);
    return maker;
  });
});

g.test('command,computePass,dispatch').
desc(
  `
Tests encoding and dispatching a simple valid compute pass on destroyed device.
  - Binds valid pipeline and bindgroups, then dispatches
  - Tests finishing encoding on destroyed device
  - Tests submitting command on destroyed device
  `
).
params((u) => u.combine('stage', kCommandValidationStages).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { stage, awaitLost } = t.params;
  const cShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('COMPUTE') });
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: { module: cShader, entryPoint: 'main' }
  });
  await t.executeCommandsAfterDestroy(stage, awaitLost, 'compute pass', (maker) => {
    maker.encoder.setPipeline(pipeline);
    maker.encoder.dispatchWorkgroups(1);
    return maker;
  });
});

g.test('command,renderPass,draw').
desc(
  `
Tests encoding and finishing a simple valid render pass on destroyed device.
  - Binds valid pipeline and bindgroups, then draws
  - Tests finishing encoding on destroyed device
  - Tests submitting command on destroyed device
  `
).
params((u) => u.combine('stage', kCommandValidationStages).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { stage, awaitLost } = t.params;
  const vShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('VERTEX') });
  const fShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('FRAGMENT') });
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module: vShader, entryPoint: 'main' },
    fragment: {
      module: fShader,
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm', writeMask: 0 }]
    }
  });
  await t.executeCommandsAfterDestroy(stage, awaitLost, 'render pass', (maker) => {
    maker.encoder.setPipeline(pipeline);
    maker.encoder.draw(0);
    return maker;
  });
});

g.test('command,renderPass,renderBundle').
desc(
  `
Tests encoding and drawing a render pass including a render bundle on destroyed device.
  - Binds valid pipeline and bindgroups, executes render bundle, then draws
  - Tests finishing encoding on destroyed device
  - Tests submitting command on destroyed device
  `
).
params((u) => u.combine('stage', kCommandValidationStages).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { stage, awaitLost } = t.params;
  const vShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('VERTEX') });
  const fShader = t.device.createShaderModule({ code: t.getNoOpShaderCode('FRAGMENT') });
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module: vShader, entryPoint: 'main' },
    fragment: {
      module: fShader,
      entryPoint: 'main',
      targets: [{ format: 'rgba8unorm', writeMask: 0 }]
    }
  });
  await t.executeCommandsAfterDestroy(stage, awaitLost, 'render bundle', (maker) => {
    maker.encoder.setPipeline(pipeline);
    maker.encoder.draw(0);
    return maker;
  });
});

g.test('queue,writeBuffer').
desc(
  `
Tests writeBuffer on queue on destroyed device.
  `
).
params((u) => u.combine('numElements', [4, 8, 16]).combine('awaitLost', [true, false])).
fn(async (t) => {
  const { numElements, awaitLost } = t.params;
  const buffer = t.device.createBuffer({
    size: numElements,
    usage: GPUBufferUsage.COPY_DST
  });
  const data = new Uint8Array(numElements);
  await t.executeAfterDestroy(() => {
    t.device.queue.writeBuffer(buffer, 0, data);
  }, awaitLost);
});

g.test('queue,writeTexture,2d,uncompressed_format').
desc(
  `
Tests writeTexture on queue on destroyed device with uncompressed formats.
  `
).
params((u) => u.combine('format', kRegularTextureFormats).combine('awaitLost', [true, false])).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
}).
fn(async (t) => {
  const { format, awaitLost } = t.params;
  const {
    blockWidth,
    blockHeight,
    color: { bytes: bytesPerBlock }
  } = kTextureFormatInfo[format];
  const data = new Uint8Array(bytesPerBlock);
  const texture = t.device.createTexture({
    size: { width: blockWidth, height: blockHeight },
    usage: GPUTextureUsage.COPY_DST,
    format
  });
  await t.executeAfterDestroy(() => {
    t.device.queue.writeTexture(
      { texture },
      data,
      {},
      { width: blockWidth, height: blockHeight }
    );
  }, awaitLost);
});

g.test('queue,writeTexture,2d,compressed_format').
desc(
  `
Tests writeTexture on queue on destroyed device with compressed formats.
  `
).
params((u) =>
u.
combine('format', kCompressedTextureFormats).

combine('awaitLost', [true, false])
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  t.selectDeviceOrSkipTestCase(kTextureFormatInfo[format].feature);
}).
fn(async (t) => {
  const { format, awaitLost } = t.params;
  const {
    blockWidth,
    blockHeight,
    color: { bytes: bytesPerBlock }
  } = kTextureFormatInfo[format];
  const data = new Uint8Array(bytesPerBlock);
  const texture = t.device.createTexture({
    size: { width: blockWidth, height: blockHeight },
    usage: GPUTextureUsage.COPY_DST,
    format
  });
  await t.executeAfterDestroy(() => {
    t.device.queue.writeTexture(
      { texture },
      data,
      {},
      { width: blockWidth, height: blockHeight }
    );
  }, awaitLost);
});

g.test('queue,copyExternalImageToTexture,canvas').
desc(
  `
Tests copyExternalImageToTexture from canvas on queue on destroyed device.
  `
).
params((u) =>
u.
combine('canvasType', kAllCanvasTypes).
combine('contextType', kValidCanvasContextIds).

combine('awaitLost', [true, false])
).
fn(async (t) => {
  const { canvasType, contextType, awaitLost } = t.params;
  const canvas = createCanvas(t, canvasType, 1, 1);
  const texture = t.device.createTexture({
    size: { width: 1, height: 1 },
    format: 'bgra8unorm',
    usage: GPUTextureUsage.COPY_DST
  });

  const ctx = canvas.getContext(contextType);
  if (ctx === null) {
    t.skip('Failed to get context for canvas element');
    return;
  }
  t.tryTrackForCleanup(ctx);

  await t.executeAfterDestroy(() => {
    t.device.queue.copyExternalImageToTexture(
      { source: canvas },
      { texture },
      { width: 1, height: 1 }
    );
  }, awaitLost);
});

g.test('queue,copyExternalImageToTexture,imageBitmap').
desc(
  `
Tests copyExternalImageToTexture from canvas on queue on destroyed device.
  `
).
params((u) => u.combine('awaitLost', [true, false])).
fn(async (t) => {
  const { awaitLost } = t.params;
  if (typeof createImageBitmap === 'undefined') {
    t.skip('Creating ImageBitmaps is not supported.');
  }
  const imageBitmap = await createImageBitmap(new ImageData(new Uint8ClampedArray(4), 1, 1));

  const texture = t.device.createTexture({
    size: { width: 1, height: 1 },
    format: 'bgra8unorm',
    usage: GPUTextureUsage.COPY_DST
  });

  await t.executeAfterDestroy(() => {
    t.device.queue.copyExternalImageToTexture(
      { source: imageBitmap },
      { texture },
      { width: 1, height: 1 }
    );
  }, awaitLost);
});