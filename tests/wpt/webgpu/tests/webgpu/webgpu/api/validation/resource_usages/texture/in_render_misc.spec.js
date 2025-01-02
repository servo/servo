/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Texture Usages Validation Tests on All Kinds of WebGPU Subresource Usage Scopes.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { unreachable } from '../../../../../common/util/util.js';
import { kTextureUsages } from '../../../../capability_info.js';
import { ValidationTest } from '../../validation_test.js';
import {

  kTextureBindingTypes,
  IsReadOnlyTextureBindingType } from
'../texture/in_render_common.spec.js';

class F extends ValidationTest {
  createBindGroupLayoutForTest(
  textureUsage,
  sampleType,
  visibility = GPUShaderStage.FRAGMENT)
  {
    const bindGroupLayoutEntry = {
      binding: 0,
      visibility
    };

    switch (textureUsage) {
      case 'sampled-texture':
        bindGroupLayoutEntry.texture = { viewDimension: '2d-array', sampleType };
        break;
      case 'readonly-storage-texture':
        bindGroupLayoutEntry.storageTexture = {
          access: 'read-only',
          format: 'r32float',
          viewDimension: '2d-array'
        };
        break;
      case 'writeonly-storage-texture':
        bindGroupLayoutEntry.storageTexture = {
          access: 'write-only',
          format: 'r32float',
          viewDimension: '2d-array'
        };
        break;
      case 'readwrite-storage-texture':
        bindGroupLayoutEntry.storageTexture = {
          access: 'read-write',
          format: 'r32float',
          viewDimension: '2d-array'
        };
        break;
      default:
        unreachable();
        break;
    }
    return this.device.createBindGroupLayout({
      entries: [bindGroupLayoutEntry]
    });
  }

  createBindGroupForTest(
  textureView,
  textureUsage,
  sampleType,
  visibility = GPUShaderStage.FRAGMENT)
  {
    return this.device.createBindGroup({
      layout: this.createBindGroupLayoutForTest(textureUsage, sampleType, visibility),
      entries: [{ binding: 0, resource: textureView }]
    });
  }
}

export const g = makeTestGroup(F);

const kTextureSize = 16;
const kTextureLayers = 3;

g.test('subresources,set_bind_group_on_same_index_color_texture').
desc(
  `
  Test that when one color texture subresource is bound to different bind groups, whether the bind
  groups are reset by another compatible ones or not, its list of internal usages within one usage
  scope can only be a compatible usage list.`
).
params((u) =>
u.
combine('useDifferentTextureAsTexture2', [true, false]).
combine('baseLayer2', [0, 1]).
combine('view1Binding', kTextureBindingTypes).
combine('view2Binding', kTextureBindingTypes)
).
fn((t) => {
  const { useDifferentTextureAsTexture2, baseLayer2, view1Binding, view2Binding } = t.params;

  const texture0 = t.createTextureTracked({
    format: 'r32float',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.STORAGE_BINDING,
    size: [kTextureSize, kTextureSize, kTextureLayers]
  });
  // We always bind the first layer of the texture to bindGroup0.
  const textureView0 = texture0.createView({
    dimension: '2d-array',
    baseArrayLayer: 0,
    arrayLayerCount: 1
  });
  const bindGroup0 = t.createBindGroupForTest(textureView0, view1Binding, 'unfilterable-float');
  const bindGroup1 = t.createBindGroupForTest(textureView0, view2Binding, 'unfilterable-float');

  const texture2 = useDifferentTextureAsTexture2 ?
  t.createTextureTracked({
    format: 'r32float',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.STORAGE_BINDING,
    size: [kTextureSize, kTextureSize, kTextureLayers]
  }) :
  texture0;
  const textureView2 = texture2.createView({
    dimension: '2d-array',
    baseArrayLayer: baseLayer2,
    arrayLayerCount: kTextureLayers - baseLayer2
  });
  // There should be no conflict between bindGroup0 and validBindGroup2.
  const validBindGroup2 = t.createBindGroupForTest(
    textureView2,
    view2Binding,
    'unfilterable-float'
  );

  const unusedColorTexture = t.createTextureTracked({
    format: 'r32float',
    usage: GPUTextureUsage.RENDER_ATTACHMENT,
    size: [kTextureSize, kTextureSize, 1]
  });
  const encoder = t.device.createCommandEncoder();
  const renderPassEncoder = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: unusedColorTexture.createView(),
      loadOp: 'load',
      storeOp: 'store'
    }]

  });
  renderPassEncoder.setBindGroup(0, bindGroup0);
  renderPassEncoder.setBindGroup(1, bindGroup1);
  renderPassEncoder.setBindGroup(1, validBindGroup2);
  renderPassEncoder.end();

  const noConflict =
  IsReadOnlyTextureBindingType(view1Binding) && IsReadOnlyTextureBindingType(view2Binding) ||
  view1Binding === view2Binding;
  t.expectValidationError(() => {
    encoder.finish();
  }, !noConflict);
});

g.test('subresources,set_bind_group_on_same_index_depth_stencil_texture').
desc(
  `
  Test that when one depth stencil texture subresource is bound to different bind groups, whether
  the bind groups are reset by another compatible ones or not, its list of internal usages within
  one usage scope can only be a compatible usage list.`
).
params((u) =>
u.
combine('bindAspect', ['depth-only', 'stencil-only']).
combine('depthStencilReadOnly', [true, false])
).
fn((t) => {
  const { bindAspect, depthStencilReadOnly } = t.params;
  const depthStencilTexture = t.createTextureTracked({
    format: 'depth24plus-stencil8',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
    size: [kTextureSize, kTextureSize, 1],
    ...(t.isCompatibility && {
      textureBindingViewDimension: '2d-array'
    })
  });

  const conflictedToNonReadOnlyAttachmentBindGroup = t.createBindGroupForTest(
    depthStencilTexture.createView({
      dimension: '2d-array',
      aspect: bindAspect
    }),
    'sampled-texture',
    bindAspect === 'depth-only' ? 'depth' : 'uint'
  );

  const colorTexture = t.createTextureTracked({
    format: 'r32float',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.STORAGE_BINDING,
    size: [kTextureSize, kTextureSize, 1],
    ...(t.isCompatibility && {
      textureBindingViewDimension: '2d-array'
    })
  });
  const validBindGroup = t.createBindGroupForTest(
    colorTexture.createView({
      dimension: '2d-array'
    }),
    'sampled-texture',
    'unfilterable-float'
  );

  const encoder = t.device.createCommandEncoder();
  const renderPassEncoder = encoder.beginRenderPass({
    colorAttachments: [],
    depthStencilAttachment: {
      view: depthStencilTexture.createView(),
      depthReadOnly: depthStencilReadOnly,
      stencilReadOnly: depthStencilReadOnly
    }
  });
  renderPassEncoder.setBindGroup(0, conflictedToNonReadOnlyAttachmentBindGroup);
  renderPassEncoder.setBindGroup(0, validBindGroup);
  renderPassEncoder.end();

  t.expectValidationError(() => {
    encoder.finish();
  }, !depthStencilReadOnly);
});

g.test('subresources,set_unused_bind_group').
desc(
  `
  Test that when one texture subresource is bound to different bind groups and the bind groups are
  used in the same render or compute pass encoder, its list of internal usages within one usage
  scope can only be a compatible usage list.`
).
params((u) =>
u.
combine('inRenderPass', [true, false]).
combine('textureUsage0', kTextureBindingTypes).
combine('textureUsage1', kTextureBindingTypes)
).
fn((t) => {
  const { inRenderPass, textureUsage0, textureUsage1 } = t.params;

  if (
  textureUsage0 === 'readwrite-storage-texture' ||
  textureUsage1 === 'readwrite-storage-texture')
  {
    t.skipIfLanguageFeatureNotSupported('readonly_and_readwrite_storage_textures');
  }

  const texture0 = t.createTextureTracked({
    format: 'r32float',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.STORAGE_BINDING,
    size: [kTextureSize, kTextureSize, kTextureLayers]
  });
  // We always bind the first layer of the texture to bindGroup0.
  const textureView0 = texture0.createView({
    dimension: '2d-array',
    baseArrayLayer: 0,
    arrayLayerCount: 1
  });
  const visibility = inRenderPass ? GPUShaderStage.FRAGMENT : GPUShaderStage.COMPUTE;
  // bindGroup0 is used by the pipelines, and bindGroup1 is not used by the pipelines.
  const bindGroup0 = t.createBindGroupForTest(
    textureView0,
    textureUsage0,
    'unfilterable-float',
    visibility
  );
  const bindGroup1 = t.createBindGroupForTest(
    textureView0,
    textureUsage1,
    'unfilterable-float',
    visibility
  );

  const encoder = t.device.createCommandEncoder();
  const colorTexture = t.createTextureTracked({
    format: 'r32float',
    usage: GPUTextureUsage.RENDER_ATTACHMENT,
    size: [kTextureSize, kTextureSize, 1]
  });
  if (inRenderPass) {
    let fragmentShader = '';
    switch (textureUsage0) {
      case 'sampled-texture':
        fragmentShader = `
          @group(0) @binding(0) var texture0 : texture_2d_array<f32>;
          @fragment fn main()
            -> @location(0) vec4<f32> {
              return textureLoad(texture0, vec2<i32>(), 0, 0);
          }
          `;
        break;
      case `readonly-storage-texture`:
        fragmentShader = `
          @group(0) @binding(0) var texture0 : texture_storage_2d_array<r32float, read>;
          @fragment fn main()
            -> @location(0) vec4<f32> {
              return textureLoad(texture0, vec2<i32>(), 0);
          }
          `;
        break;
      case `writeonly-storage-texture`:
        fragmentShader = `
            @group(0) @binding(0) var texture0 : texture_storage_2d_array<r32float, write>;
            @fragment fn main()
              -> @location(0) vec4<f32> {
                textureStore(texture0, vec2i(), 0, vec4f(1, 0, 0, 1));
                return vec4f(0, 0, 0, 1);
            }
            `;
        break;
      case `readwrite-storage-texture`:
        fragmentShader = `
            @group(0) @binding(0) var texture0 : texture_storage_2d_array<r32float, read_write>;
            @fragment fn main()
              -> @location(0) vec4<f32> {
                let color = textureLoad(texture0, vec2i(), 0);
                textureStore(texture0, vec2i(), 0, vec4f(1, 0, 0, 1));
                return color;
            }
            `;
        break;
    }

    const renderPipeline = t.device.createRenderPipeline({
      layout: t.device.createPipelineLayout({
        bindGroupLayouts: [
        t.createBindGroupLayoutForTest(textureUsage0, 'unfilterable-float', visibility)]

      }),
      vertex: {
        module: t.device.createShaderModule({
          code: t.getNoOpShaderCode('VERTEX')
        })
      },
      fragment: {
        module: t.device.createShaderModule({
          code: fragmentShader
        }),
        targets: [{ format: 'r32float' }]
      }
    });

    const renderPassEncoder = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorTexture.createView(),
        loadOp: 'load',
        storeOp: 'store'
      }]

    });
    renderPassEncoder.setBindGroup(0, bindGroup0);
    renderPassEncoder.setBindGroup(1, bindGroup1);
    renderPassEncoder.setPipeline(renderPipeline);
    renderPassEncoder.draw(1);
    renderPassEncoder.end();
  } else {
    let computeShader = '';
    switch (textureUsage0) {
      case 'sampled-texture':
        computeShader = `
          @group(0) @binding(0) var texture0 : texture_2d_array<f32>;
          @group(1) @binding(0) var writableStorage : texture_storage_2d_array<r32float, write>;
          @compute @workgroup_size(1) fn main() {
              let value = textureLoad(texture0, vec2i(), 0, 0);
              textureStore(writableStorage, vec2i(), 0, value);
          }
          `;
        break;
      case `readonly-storage-texture`:
        computeShader = `
          @group(0) @binding(0) var texture0 : texture_storage_2d_array<r32float, read>;
          @group(1) @binding(0) var writableStorage : texture_storage_2d_array<r32float, write>;
          @compute @workgroup_size(1) fn main() {
              let value = textureLoad(texture0, vec2<i32>(), 0);
              textureStore(writableStorage, vec2i(), 0, value);
          }
          `;
        break;
      case `writeonly-storage-texture`:
        computeShader = `
            @group(0) @binding(0) var texture0 : texture_storage_2d_array<r32float, write>;
            @group(1) @binding(0) var writableStorage : texture_storage_2d_array<r32float, write>;
            @compute @workgroup_size(1) fn main() {
                textureStore(texture0, vec2i(), 0, vec4f(1, 0, 0, 1));
                textureStore(writableStorage, vec2i(), 0, vec4f(1, 0, 0, 1));
            }
            `;
        break;
      case `readwrite-storage-texture`:
        computeShader = `
            @group(0) @binding(0) var texture0 : texture_storage_2d_array<r32float, read_write>;
            @group(1) @binding(0) var writableStorage : texture_storage_2d_array<r32float, write>;
            @compute @workgroup_size(1) fn main() {
                let color = textureLoad(texture0, vec2i(), 0);
                textureStore(texture0, vec2i(), 0, vec4f(1, 0, 0, 1));
                textureStore(writableStorage, vec2i(), 0, color);
            }
            `;
        break;
    }

    const pipelineLayout = t.device.createPipelineLayout({
      bindGroupLayouts: [
      t.createBindGroupLayoutForTest(textureUsage0, 'unfilterable-float', visibility),
      t.createBindGroupLayoutForTest(
        'writeonly-storage-texture',
        'unfilterable-float',
        visibility
      )]

    });
    const computePipeline = t.device.createComputePipeline({
      layout: pipelineLayout,
      compute: {
        module: t.device.createShaderModule({
          code: computeShader
        })
      }
    });

    const writableStorageTexture = t.createTextureTracked({
      format: 'r32float',
      usage: GPUTextureUsage.STORAGE_BINDING,
      size: [kTextureSize, kTextureSize, 1]
    });
    const writableStorageTextureView = writableStorageTexture.createView({
      dimension: '2d-array',
      baseArrayLayer: 0,
      arrayLayerCount: 1
    });
    const writableStorageTextureBindGroup = t.createBindGroupForTest(
      writableStorageTextureView,
      'writeonly-storage-texture',
      'unfilterable-float',
      visibility
    );

    const computePassEncoder = encoder.beginComputePass();
    computePassEncoder.setBindGroup(0, bindGroup0);
    computePassEncoder.setBindGroup(1, writableStorageTextureBindGroup);
    computePassEncoder.setBindGroup(2, bindGroup1);
    computePassEncoder.setPipeline(computePipeline);
    computePassEncoder.dispatchWorkgroups(1);
    computePassEncoder.end();
  }

  // In WebGPU SPEC (https://gpuweb.github.io/gpuweb/#programming-model-synchronization):
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
  const success =
  !inRenderPass ||
  IsReadOnlyTextureBindingType(textureUsage0) &&
  IsReadOnlyTextureBindingType(textureUsage1) ||
  textureUsage0 === textureUsage1;
  t.expectValidationError(() => {
    encoder.finish();
  }, !success);
});

g.test('subresources,texture_usages_in_copy_and_render_pass').
desc(
  `
  Test that using one texture subresource in a render pass encoder and a copy command is always
  allowed as WebGPU SPEC (chapter 3.4.5) defines that out of any pass encoder, each command always
  belongs to one usage scope.`
).
params((u) =>
u.
combine('usage0', [
'copy-src',
'copy-dst',
'color-attachment',
...kTextureBindingTypes]
).
combine('usage1', [
'copy-src',
'copy-dst',
'color-attachment',
...kTextureBindingTypes]
).
filter(
  ({ usage0, usage1 }) =>
  usage0 === 'copy-src' ||
  usage0 === 'copy-dst' ||
  usage1 === 'copy-src' ||
  usage1 === 'copy-dst'
)
).
fn((t) => {
  const { usage0, usage1 } = t.params;

  const texture = t.createTextureTracked({
    format: 'r32float',
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.COPY_DST |
    GPUTextureUsage.TEXTURE_BINDING |
    GPUTextureUsage.STORAGE_BINDING |
    GPUTextureUsage.RENDER_ATTACHMENT,
    size: [kTextureSize, kTextureSize, 1],
    ...(t.isCompatibility && {
      textureBindingViewDimension: '2d-array'
    })
  });

  const UseTextureOnCommandEncoder = (
  texture,
  usage,
  encoder) =>
  {
    switch (usage) {
      case 'copy-src':{
          const buffer = t.createBufferWithState('valid', {
            size: 4,
            usage: GPUBufferUsage.COPY_DST
          });
          encoder.copyTextureToBuffer({ texture }, { buffer }, [1, 1, 1]);
          break;
        }
      case 'copy-dst':{
          const buffer = t.createBufferWithState('valid', {
            size: 4,
            usage: GPUBufferUsage.COPY_SRC
          });
          encoder.copyBufferToTexture({ buffer }, { texture }, [1, 1, 1]);
          break;
        }
      case 'color-attachment':{
          const renderPassEncoder = encoder.beginRenderPass({
            colorAttachments: [{ view: texture.createView(), loadOp: 'load', storeOp: 'store' }]
          });
          renderPassEncoder.end();
          break;
        }
      case 'sampled-texture':
      case 'readonly-storage-texture':
      case 'writeonly-storage-texture':
      case 'readwrite-storage-texture':{
          const colorTexture = t.createTextureTracked({
            format: 'r32float',
            usage: GPUTextureUsage.RENDER_ATTACHMENT,
            size: [kTextureSize, kTextureSize, 1]
          });
          const renderPassEncoder = encoder.beginRenderPass({
            colorAttachments: [
            { view: colorTexture.createView(), loadOp: 'load', storeOp: 'store' }]

          });
          const bindGroup = t.createBindGroupForTest(
            texture.createView({
              dimension: '2d-array'
            }),
            usage,
            'unfilterable-float'
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

g.test('subresources,texture_view_usages').
desc(
  `
  Test that the usages of the texture view are used to validate compatibility in command encoding
  instead of the usages of the base texture.`
).
params((u) =>
u.
combine('bindingType', ['color-attachment', ...kTextureBindingTypes]).
combine('viewUsage', [0, ...kTextureUsages])
).
fn((t) => {
  const { bindingType, viewUsage } = t.params;

  const texture = t.createTextureTracked({
    format: 'r32float',
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.COPY_DST |
    GPUTextureUsage.TEXTURE_BINDING |
    GPUTextureUsage.STORAGE_BINDING |
    GPUTextureUsage.RENDER_ATTACHMENT,
    size: [kTextureSize, kTextureSize, 1],
    ...(t.isCompatibility && {
      textureBindingViewDimension: '2d-array'
    })
  });

  switch (bindingType) {
    case 'color-attachment':{
        const encoder = t.device.createCommandEncoder();
        const renderPassEncoder = encoder.beginRenderPass({
          colorAttachments: [
          { view: texture.createView({ usage: viewUsage }), loadOp: 'load', storeOp: 'store' }]

        });
        renderPassEncoder.end();

        const success = viewUsage === 0 || (viewUsage & GPUTextureUsage.RENDER_ATTACHMENT) !== 0;

        t.expectValidationError(() => {
          encoder.finish();
        }, !success);
        break;
      }
    case 'sampled-texture':
    case 'readonly-storage-texture':
    case 'writeonly-storage-texture':
    case 'readwrite-storage-texture':
      {
        let success = true;
        if (viewUsage !== 0) {
          if (bindingType === 'sampled-texture') {
            if ((viewUsage & GPUTextureUsage.TEXTURE_BINDING) === 0) success = false;
          } else {
            if ((viewUsage & GPUTextureUsage.STORAGE_BINDING) === 0) success = false;
          }
        }

        t.expectValidationError(() => {
          t.createBindGroupForTest(
            texture.createView({
              dimension: '2d-array',
              usage: viewUsage
            }),
            bindingType,
            'unfilterable-float'
          );
        }, !success);
      }
      break;
    default:
      unreachable();
  }
});