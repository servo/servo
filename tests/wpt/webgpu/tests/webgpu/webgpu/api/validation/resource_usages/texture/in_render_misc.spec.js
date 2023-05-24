/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Texture Usages Validation Tests on All Kinds of WebGPU Subresource Usage Scopes.
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { unreachable } from '../../../../../common/util/util.js';
import { ValidationTest } from '../../validation_test.js';

class F extends ValidationTest {
  createBindGroupLayoutForTest(textureUsage, sampleType, visibility = GPUShaderStage['FRAGMENT']) {
    const bindGroupLayoutEntry = {
      binding: 0,
      visibility,
    };

    switch (textureUsage) {
      case 'texture':
        bindGroupLayoutEntry.texture = { viewDimension: '2d-array', sampleType };
        break;
      case 'storage':
        bindGroupLayoutEntry.storageTexture = {
          access: 'write-only',
          format: 'rgba8unorm',
          viewDimension: '2d-array',
        };
        break;
      default:
        unreachable();
        break;
    }

    return this.device.createBindGroupLayout({
      entries: [bindGroupLayoutEntry],
    });
  }

  createBindGroupForTest(
    textureView,
    textureUsage,
    sampleType,
    visibility = GPUShaderStage['FRAGMENT']
  ) {
    return this.device.createBindGroup({
      layout: this.createBindGroupLayoutForTest(textureUsage, sampleType, visibility),
      entries: [{ binding: 0, resource: textureView }],
    });
  }
}

export const g = makeTestGroup(F);

const kTextureSize = 16;
const kTextureLayers = 3;

g.test('subresources,set_bind_group_on_same_index_color_texture')
  .desc(
    `
  Test that when one color texture subresource is bound to different bind groups, whether the bind
  groups are reset by another compatible ones or not, its list of internal usages within one usage
  scope can only be a compatible usage list.`
  )
  .params(u =>
    u
      .combineWithParams([
        { useDifferentTextureAsTexture2: true, baseLayer2: 0, view2Binding: 'texture' },
        { useDifferentTextureAsTexture2: false, baseLayer2: 0, view2Binding: 'texture' },
        { useDifferentTextureAsTexture2: false, baseLayer2: 1, view2Binding: 'texture' },
        { useDifferentTextureAsTexture2: false, baseLayer2: 0, view2Binding: 'storage' },
        { useDifferentTextureAsTexture2: false, baseLayer2: 1, view2Binding: 'storage' },
      ])
      .combine('hasConflict', [true, false])
  )
  .fn(t => {
    const { useDifferentTextureAsTexture2, baseLayer2, view2Binding, hasConflict } = t.params;

    const texture0 = t.device.createTexture({
      format: 'rgba8unorm',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.STORAGE_BINDING,
      size: [kTextureSize, kTextureSize, kTextureLayers],
    });
    // We always bind the first layer of the texture to bindGroup0.
    const textureView0 = texture0.createView({
      dimension: '2d-array',
      baseArrayLayer: 0,
      arrayLayerCount: 1,
    });
    const bindGroup0 = t.createBindGroupForTest(textureView0, view2Binding, 'float');

    // In one renderPassEncoder it is an error to set both bindGroup0 and bindGroup1.
    const view1Binding = hasConflict
      ? view2Binding === 'texture'
        ? 'storage'
        : 'texture'
      : view2Binding;
    const bindGroup1 = t.createBindGroupForTest(textureView0, view1Binding, 'float');

    const texture2 = useDifferentTextureAsTexture2
      ? t.device.createTexture({
          format: 'rgba8unorm',
          usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.STORAGE_BINDING,
          size: [kTextureSize, kTextureSize, kTextureLayers],
        })
      : texture0;
    const textureView2 = texture2.createView({
      dimension: '2d-array',
      baseArrayLayer: baseLayer2,
      arrayLayerCount: kTextureLayers - baseLayer2,
    });
    // There should be no conflict between bindGroup0 and validBindGroup2.
    const validBindGroup2 = t.createBindGroupForTest(textureView2, view2Binding, 'float');

    const colorTexture = t.device.createTexture({
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
      size: [kTextureSize, kTextureSize, 1],
    });
    const encoder = t.device.createCommandEncoder();
    const renderPassEncoder = encoder.beginRenderPass({
      colorAttachments: [
        {
          view: colorTexture.createView(),
          loadOp: 'load',
          storeOp: 'store',
        },
      ],
    });
    renderPassEncoder.setBindGroup(0, bindGroup0);
    renderPassEncoder.setBindGroup(1, bindGroup1);
    renderPassEncoder.setBindGroup(1, validBindGroup2);
    renderPassEncoder.end();

    t.expectValidationError(() => {
      encoder.finish();
    }, hasConflict);
  });

g.test('subresources,set_bind_group_on_same_index_depth_stencil_texture')
  .desc(
    `
  Test that when one depth stencil texture subresource is bound to different bind groups, whether
  the bind groups are reset by another compatible ones or not, its list of internal usages within
  one usage scope can only be a compatible usage list.`
  )
  .params(u =>
    u
      .combine('bindAspect', ['depth-only', 'stencil-only'])
      .combine('depthStencilReadOnly', [true, false])
  )
  .fn(t => {
    const { bindAspect, depthStencilReadOnly } = t.params;
    const depthStencilTexture = t.device.createTexture({
      format: 'depth24plus-stencil8',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
      size: [kTextureSize, kTextureSize, 1],
    });

    const conflictedToNonReadOnlyAttachmentBindGroup = t.createBindGroupForTest(
      depthStencilTexture.createView({
        dimension: '2d-array',
        aspect: bindAspect,
      }),
      'texture',
      bindAspect === 'depth-only' ? 'depth' : 'uint'
    );

    const colorTexture = t.device.createTexture({
      format: 'rgba8unorm',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.STORAGE_BINDING,
      size: [kTextureSize, kTextureSize, 1],
    });
    const validBindGroup = t.createBindGroupForTest(
      colorTexture.createView({
        dimension: '2d-array',
      }),
      'texture',
      'float'
    );

    const encoder = t.device.createCommandEncoder();
    const renderPassEncoder = encoder.beginRenderPass({
      colorAttachments: [],
      depthStencilAttachment: {
        view: depthStencilTexture.createView(),
        depthReadOnly: depthStencilReadOnly,
        stencilReadOnly: depthStencilReadOnly,
      },
    });
    renderPassEncoder.setBindGroup(0, conflictedToNonReadOnlyAttachmentBindGroup);
    renderPassEncoder.setBindGroup(0, validBindGroup);
    renderPassEncoder.end();

    t.expectValidationError(() => {
      encoder.finish();
    }, !depthStencilReadOnly);
  });

g.test('subresources,set_unused_bind_group')
  .desc(
    `
  Test that when one texture subresource is bound to different bind groups and the bind groups are
  used in the same render or compute pass encoder, its list of internal usages within one usage
  scope can only be a compatible usage list.`
  )
  .params(u => u.combine('inRenderPass', [true, false]).combine('hasConflict', [true, false]))
  .fn(t => {
    const { inRenderPass, hasConflict } = t.params;

    const texture0 = t.device.createTexture({
      format: 'rgba8unorm',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.STORAGE_BINDING,
      size: [kTextureSize, kTextureSize, kTextureLayers],
    });
    // We always bind the first layer of the texture to bindGroup0.
    const textureView0 = texture0.createView({
      dimension: '2d-array',
      baseArrayLayer: 0,
      arrayLayerCount: 1,
    });
    const visibility = inRenderPass ? GPUShaderStage.FRAGMENT : GPUShaderStage.COMPUTE;
    // bindGroup0 is used by the pipelines, and bindGroup1 is not used by the pipelines.
    const textureUsage0 = inRenderPass ? 'texture' : 'storage';
    const textureUsage1 = hasConflict ? (inRenderPass ? 'storage' : 'texture') : textureUsage0;
    const bindGroup0 = t.createBindGroupForTest(textureView0, textureUsage0, 'float', visibility);
    const bindGroup1 = t.createBindGroupForTest(textureView0, textureUsage1, 'float', visibility);

    const encoder = t.device.createCommandEncoder();
    const colorTexture = t.device.createTexture({
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
      size: [kTextureSize, kTextureSize, 1],
    });
    const pipelineLayout = t.device.createPipelineLayout({
      bindGroupLayouts: [t.createBindGroupLayoutForTest(textureUsage0, 'float', visibility)],
    });
    if (inRenderPass) {
      const renderPipeline = t.device.createRenderPipeline({
        layout: pipelineLayout,
        vertex: {
          module: t.device.createShaderModule({
            code: t.getNoOpShaderCode('VERTEX'),
          }),
          entryPoint: 'main',
        },
        fragment: {
          module: t.device.createShaderModule({
            code: `
              @group(0) @binding(0) var texture0 : texture_2d_array<f32>;
              @fragment fn main()
                -> @location(0) vec4<f32> {
                  return textureLoad(texture0, vec2<i32>(), 0, 0);
              }`,
          }),
          entryPoint: 'main',
          targets: [{ format: 'rgba8unorm' }],
        },
      });

      const renderPassEncoder = encoder.beginRenderPass({
        colorAttachments: [
          {
            view: colorTexture.createView(),
            loadOp: 'load',
            storeOp: 'store',
          },
        ],
      });
      renderPassEncoder.setBindGroup(0, bindGroup0);
      renderPassEncoder.setBindGroup(1, bindGroup1);
      renderPassEncoder.setPipeline(renderPipeline);
      renderPassEncoder.draw(1);
      renderPassEncoder.end();
    } else {
      const computePipeline = t.device.createComputePipeline({
        layout: pipelineLayout,
        compute: {
          module: t.device.createShaderModule({
            code: `
            @group(0) @binding(0) var texture0 : texture_storage_2d_array<rgba8unorm, write>;
            @compute @workgroup_size(1)
            fn main() {
              textureStore(texture0, vec2<i32>(), 0, vec4<f32>());
            }`,
          }),
          entryPoint: 'main',
        },
      });
      const computePassEncoder = encoder.beginComputePass();
      computePassEncoder.setBindGroup(0, bindGroup0);
      computePassEncoder.setBindGroup(1, bindGroup1);
      computePassEncoder.setPipeline(computePipeline);
      computePassEncoder.dispatchWorkgroups(1);
      computePassEncoder.end();
    }

    // In WebGPU SPEC (Chapter 3.4.5, Synchronization):
    // This specification defines the following usage scopes:
    // - In a compute pass, each dispatch command (dispatchWorkgroups() or
    //   dispatchWorkgroupsIndirect()) is one usage scope. A subresource is "used" in the usage
    //   scope if it is potentially accessible by the command. State-setting compute pass commands,
    //   like setBindGroup(index, bindGroup, dynamicOffsets), do not contribute directly to a usage
    //   scope.
    // - One render pass is one usage scope. A subresource is "used" in the usage scope if it’s
    //   referenced by any (state-setting or non-state-setting) command. For example, in
    //   setBindGroup(index, bindGroup, dynamicOffsets), every subresource in bindGroup is "used" in
    //   the render pass’s usage scope.
    const success = !inRenderPass || !hasConflict;
    t.expectValidationError(() => {
      encoder.finish();
    }, !success);
  });

g.test('subresources,texture_usages_in_copy_and_render_pass')
  .desc(
    `
  Test that using one texture subresource in a render pass encoder and a copy command is always
  allowed as WebGPU SPEC (chapter 3.4.5) defines that out of any pass encoder, each command always
  belongs to one usage scope.`
  )
  .params(u =>
    u
      .combine('usage0', ['copy-src', 'copy-dst', 'texture', 'storage', 'color-attachment'])
      .combine('usage1', ['copy-src', 'copy-dst', 'texture', 'storage', 'color-attachment'])
      .filter(
        ({ usage0, usage1 }) =>
          usage0 === 'copy-src' ||
          usage0 === 'copy-dst' ||
          usage1 === 'copy-src' ||
          usage1 === 'copy-dst'
      )
  )
  .fn(t => {
    const { usage0, usage1 } = t.params;

    const texture = t.device.createTexture({
      format: 'rgba8unorm',
      usage:
        GPUTextureUsage.COPY_SRC |
        GPUTextureUsage.COPY_DST |
        GPUTextureUsage.TEXTURE_BINDING |
        GPUTextureUsage.STORAGE_BINDING |
        GPUTextureUsage.RENDER_ATTACHMENT,
      size: [kTextureSize, kTextureSize, 1],
    });

    const UseTextureOnCommandEncoder = (texture, usage, encoder) => {
      switch (usage) {
        case 'copy-src': {
          const buffer = t.createBufferWithState('valid', {
            size: 4,
            usage: GPUBufferUsage.COPY_DST,
          });
          encoder.copyTextureToBuffer({ texture }, { buffer }, [1, 1, 1]);
          break;
        }
        case 'copy-dst': {
          const buffer = t.createBufferWithState('valid', {
            size: 4,
            usage: GPUBufferUsage.COPY_SRC,
          });
          encoder.copyBufferToTexture({ buffer }, { texture }, [1, 1, 1]);
          break;
        }
        case 'color-attachment': {
          const renderPassEncoder = encoder.beginRenderPass({
            colorAttachments: [{ view: texture.createView(), loadOp: 'load', storeOp: 'store' }],
          });
          renderPassEncoder.end();
          break;
        }
        case 'texture':
        case 'storage': {
          const colorTexture = t.device.createTexture({
            format: 'rgba8unorm',
            usage: GPUTextureUsage.RENDER_ATTACHMENT,
            size: [kTextureSize, kTextureSize, 1],
          });
          const renderPassEncoder = encoder.beginRenderPass({
            colorAttachments: [
              { view: colorTexture.createView(), loadOp: 'load', storeOp: 'store' },
            ],
          });
          const bindGroup = t.createBindGroupForTest(
            texture.createView({
              dimension: '2d-array',
            }),
            usage,
            'float'
          );

          renderPassEncoder.setBindGroup(0, bindGroup);
          renderPassEncoder.end();
          break;
        }
      }
    };
    const encoder = t.device.createCommandEncoder();
    UseTextureOnCommandEncoder(texture, usage0, encoder);
    UseTextureOnCommandEncoder(texture, usage1, encoder);
    t.expectValidationError(() => {
      encoder.finish();
    }, false);
  });
