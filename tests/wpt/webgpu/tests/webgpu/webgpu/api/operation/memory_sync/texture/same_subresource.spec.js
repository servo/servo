/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Memory Synchronization Tests for Texture: read before write, read after write, and write after write to the same subresource.

- TODO: Test synchronization between multiple queues.
- TODO: Test depth/stencil attachments.
- TODO: Use non-solid-color texture contents [2]
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { assert, memcpy, unreachable } from '../../../../../common/util/util.js';

import { GPUTest } from '../../../../gpu_test.js';
import { align } from '../../../../util/math.js';
import { getTextureCopyLayout } from '../../../../util/texture/layout.js';
import {
  kTexelRepresentationInfo } from

'../../../../util/texture/texel_data.js';
import {
  kOperationBoundaries,

  kBoundaryInfo,
  OperationContextHelper } from
'../operation_context_helper.js';

import {
  kAllReadOps,
  kAllWriteOps,
  checkOpsValidForContext,

  kOpInfo } from
'./texture_sync_test.js';

export const g = makeTestGroup(GPUTest);

const fullscreenQuadWGSL = `
  struct VertexOutput {
    @builtin(position) Position : vec4<f32>
  };

  @vertex fn vert_main(@builtin(vertex_index) VertexIndex : u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>( 1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(-1.0,  1.0));

    var output : VertexOutput;
    output.Position = vec4<f32>(pos[VertexIndex], 0.0, 1.0);
    return output;
  }
`;

class TextureSyncTestHelper extends OperationContextHelper {


  kTextureSize = [4, 4];
  kTextureFormat = 'rgba8unorm';

  constructor(
  t,
  textureCreationParams)


  {
    super(t);
    this.texture = t.createTextureTracked({
      size: this.kTextureSize,
      format: this.kTextureFormat,
      ...textureCreationParams
    });
  }

  /**
   * Perform a read operation on the test texture.
   * @return GPUTexture copy containing the contents.
   */
  performReadOp({ op, in: context }) {
    this.ensureContext(context);
    switch (op) {
      case 't2t-copy':{
          const texture = this.t.createTextureTracked({
            size: this.kTextureSize,
            format: this.kTextureFormat,
            usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
          });

          assert(this.commandEncoder !== undefined);
          this.commandEncoder.copyTextureToTexture(
            {
              texture: this.texture
            },
            { texture },
            this.kTextureSize
          );
          return texture;
        }
      case 't2b-copy':{
          const { byteLength, bytesPerRow } = getTextureCopyLayout(this.kTextureFormat, '2d', [
          ...this.kTextureSize,
          1]
          );
          const buffer = this.t.createBufferTracked({
            size: byteLength,
            usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
          });

          const texture = this.t.createTextureTracked({
            size: this.kTextureSize,
            format: this.kTextureFormat,
            usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
          });

          assert(this.commandEncoder !== undefined);
          this.commandEncoder.copyTextureToBuffer(
            {
              texture: this.texture
            },
            { buffer, bytesPerRow },
            this.kTextureSize
          );
          this.commandEncoder.copyBufferToTexture(
            { buffer, bytesPerRow },
            { texture },
            this.kTextureSize
          );
          return texture;
        }
      case 'sample':{
          const texture = this.t.createTextureTracked({
            size: this.kTextureSize,
            format: this.kTextureFormat,
            usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.STORAGE_BINDING
          });

          const bindGroupLayout = this.device.createBindGroupLayout({
            entries: [
            {
              binding: 0,
              visibility: GPUShaderStage.FRAGMENT | GPUShaderStage.COMPUTE,
              texture: {
                sampleType: 'unfilterable-float'
              }
            },
            {
              binding: 1,
              visibility: GPUShaderStage.FRAGMENT | GPUShaderStage.COMPUTE,
              storageTexture: {
                access: 'write-only',
                format: this.kTextureFormat
              }
            }]

          });

          const bindGroup = this.device.createBindGroup({
            layout: bindGroupLayout,
            entries: [
            {
              binding: 0,
              resource: this.texture.createView()
            },
            {
              binding: 1,
              resource: texture.createView()
            }]

          });

          switch (context) {
            case 'render-pass-encoder':
            case 'render-bundle-encoder':{
                const module = this.device.createShaderModule({
                  code: `${fullscreenQuadWGSL}

                @group(0) @binding(0) var inputTex: texture_2d<f32>;
                @group(0) @binding(1) var outputTex: texture_storage_2d<rgba8unorm, write>;

                @fragment fn frag_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
                  let coord = vec2<i32>(fragCoord.xy);
                  textureStore(outputTex, coord, textureLoad(inputTex, coord, 0));
                  return vec4<f32>();
                }
              `
                });
                const renderPipeline = this.device.createRenderPipeline({
                  layout: this.device.createPipelineLayout({
                    bindGroupLayouts: [bindGroupLayout]
                  }),
                  vertex: {
                    module,
                    entryPoint: 'vert_main'
                  },
                  fragment: {
                    module,
                    entryPoint: 'frag_main',

                    // Unused attachment since we can't use textureStore in the vertex shader.
                    // Set writeMask to zero.
                    targets: [
                    {
                      format: this.kTextureFormat,
                      writeMask: 0
                    }]

                  }
                });

                switch (context) {
                  case 'render-bundle-encoder':
                    assert(this.renderBundleEncoder !== undefined);
                    this.renderBundleEncoder.setPipeline(renderPipeline);
                    this.renderBundleEncoder.setBindGroup(0, bindGroup);
                    this.renderBundleEncoder.draw(6);
                    break;
                  case 'render-pass-encoder':
                    assert(this.renderPassEncoder !== undefined);
                    this.renderPassEncoder.setPipeline(renderPipeline);
                    this.renderPassEncoder.setBindGroup(0, bindGroup);
                    this.renderPassEncoder.draw(6);
                    break;
                }
                break;
              }
            case 'compute-pass-encoder':{
                const module = this.device.createShaderModule({
                  code: `
                @group(0) @binding(0) var inputTex: texture_2d<f32>;
                @group(0) @binding(1) var outputTex: texture_storage_2d<rgba8unorm, write>;

                @compute @workgroup_size(8, 8)
                fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
                  if (any(gid.xy >= vec2<u32>(textureDimensions(inputTex)))) {
                    return;
                  }
                  let coord = vec2<i32>(gid.xy);
                  textureStore(outputTex, coord, textureLoad(inputTex, coord, 0));
                }
              `
                });
                const computePipeline = this.device.createComputePipeline({
                  layout: this.device.createPipelineLayout({
                    bindGroupLayouts: [bindGroupLayout]
                  }),
                  compute: {
                    module,
                    entryPoint: 'main'
                  }
                });

                assert(this.computePassEncoder !== undefined);
                this.computePassEncoder.setPipeline(computePipeline);
                this.computePassEncoder.setBindGroup(0, bindGroup);
                this.computePassEncoder.dispatchWorkgroups(
                  Math.ceil(this.kTextureSize[0] / 8),
                  Math.ceil(this.kTextureSize[1] / 8)
                );
                break;
              }
            default:
              unreachable();
          }

          return texture;
        }
      case 'b2t-copy':
      case 'attachment-resolve':
      case 'attachment-store':
        unreachable();
    }
    unreachable();
  }

  performWriteOp(
  { op, in: context },
  data)
  {
    this.ensureContext(context);
    switch (op) {
      case 'attachment-store':{
          assert(this.commandEncoder !== undefined);
          this.renderPassEncoder = this.commandEncoder.beginRenderPass({
            colorAttachments: [
            {
              view: this.texture.createView(),
              // [2] Use non-solid-color texture values
              clearValue: [data.R ?? 0, data.G ?? 0, data.B ?? 0, data.A ?? 0],
              loadOp: 'clear',
              storeOp: 'store'
            }]

          });
          this.currentContext = 'render-pass-encoder';
          break;
        }
      case 'write-texture':{
          // [2] Use non-solid-color texture values
          const rep = kTexelRepresentationInfo[this.kTextureFormat];
          const texelData = rep.pack(rep.encode(data));
          const numTexels = this.kTextureSize[0] * this.kTextureSize[1];
          const fullTexelData = new ArrayBuffer(texelData.byteLength * numTexels);
          for (let i = 0; i < numTexels; ++i) {
            memcpy({ src: texelData }, { dst: fullTexelData, start: i * texelData.byteLength });
          }

          this.queue.writeTexture(
            { texture: this.texture },
            fullTexelData,
            {
              bytesPerRow: texelData.byteLength * this.kTextureSize[0]
            },
            this.kTextureSize
          );
          break;
        }
      case 't2t-copy':{
          const texture = this.t.createTextureTracked({
            size: this.kTextureSize,
            format: this.kTextureFormat,
            usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
          });

          // [2] Use non-solid-color texture values
          const rep = kTexelRepresentationInfo[this.kTextureFormat];
          const texelData = rep.pack(rep.encode(data));
          const numTexels = this.kTextureSize[0] * this.kTextureSize[1];
          const fullTexelData = new ArrayBuffer(texelData.byteLength * numTexels);
          for (let i = 0; i < numTexels; ++i) {
            memcpy({ src: texelData }, { dst: fullTexelData, start: i * texelData.byteLength });
          }

          this.queue.writeTexture(
            { texture },
            fullTexelData,
            {
              bytesPerRow: texelData.byteLength * this.kTextureSize[0]
            },
            this.kTextureSize
          );

          assert(this.commandEncoder !== undefined);
          this.commandEncoder.copyTextureToTexture(
            { texture },
            { texture: this.texture },
            this.kTextureSize
          );
          break;
        }
      case 'b2t-copy':{
          // [2] Use non-solid-color texture values
          const rep = kTexelRepresentationInfo[this.kTextureFormat];
          const texelData = rep.pack(rep.encode(data));
          const bytesPerRow = align(texelData.byteLength, 256);
          const fullTexelData = new ArrayBuffer(bytesPerRow * this.kTextureSize[1]);
          for (let i = 0; i < this.kTextureSize[1]; ++i) {
            for (let j = 0; j < this.kTextureSize[0]; ++j) {
              memcpy(
                { src: texelData },
                {
                  dst: fullTexelData,
                  start: i * bytesPerRow + j * texelData.byteLength
                }
              );
            }
          }

          const buffer = this.t.createBufferTracked({
            size: fullTexelData.byteLength,
            usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
          });

          this.queue.writeBuffer(buffer, 0, fullTexelData);

          assert(this.commandEncoder !== undefined);
          this.commandEncoder.copyBufferToTexture(
            { buffer, bytesPerRow },
            { texture: this.texture },
            this.kTextureSize
          );
          break;
        }
      case 'attachment-resolve':{
          assert(this.commandEncoder !== undefined);
          const renderTarget = this.t.createTextureTracked({
            format: this.kTextureFormat,
            size: this.kTextureSize,
            usage: GPUTextureUsage.RENDER_ATTACHMENT,
            sampleCount: 4
          });
          this.renderPassEncoder = this.commandEncoder.beginRenderPass({
            colorAttachments: [
            {
              view: renderTarget.createView(),
              resolveTarget: this.texture.createView(),
              // [2] Use non-solid-color texture values
              clearValue: [data.R ?? 0, data.G ?? 0, data.B ?? 0, data.A ?? 0],
              loadOp: 'clear',
              storeOp: 'discard'
            }]

          });
          this.currentContext = 'render-pass-encoder';
          break;
        }
      case 'storage':{
          const bindGroupLayout = this.device.createBindGroupLayout({
            entries: [
            {
              binding: 0,
              visibility: GPUShaderStage.FRAGMENT | GPUShaderStage.COMPUTE,
              storageTexture: {
                access: 'write-only',
                format: this.kTextureFormat
              }
            }]

          });

          const bindGroup = this.device.createBindGroup({
            layout: bindGroupLayout,
            entries: [
            {
              binding: 0,
              resource: this.texture.createView()
            }]

          });

          // [2] Use non-solid-color texture values
          const storedValue = `vec4<f32>(${[data.R ?? 0, data.G ?? 0, data.B ?? 0, data.A ?? 0].
          map((x) => x.toFixed(5)).
          join(', ')})`;

          switch (context) {
            case 'render-pass-encoder':
            case 'render-bundle-encoder':{
                const module = this.device.createShaderModule({
                  code: `${fullscreenQuadWGSL}

                @group(0) @binding(0) var outputTex: texture_storage_2d<rgba8unorm, write>;

                @fragment fn frag_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
                  textureStore(outputTex, vec2<i32>(fragCoord.xy), ${storedValue});
                  return vec4<f32>();
                }
              `
                });
                const renderPipeline = this.device.createRenderPipeline({
                  layout: this.device.createPipelineLayout({
                    bindGroupLayouts: [bindGroupLayout]
                  }),
                  vertex: {
                    module,
                    entryPoint: 'vert_main'
                  },
                  fragment: {
                    module,
                    entryPoint: 'frag_main',

                    // Unused attachment since we can't use textureStore in the vertex shader.
                    // Set writeMask to zero.
                    targets: [
                    {
                      format: this.kTextureFormat,
                      writeMask: 0
                    }]

                  }
                });

                switch (context) {
                  case 'render-bundle-encoder':
                    assert(this.renderBundleEncoder !== undefined);
                    this.renderBundleEncoder.setPipeline(renderPipeline);
                    this.renderBundleEncoder.setBindGroup(0, bindGroup);
                    this.renderBundleEncoder.draw(6);
                    break;
                  case 'render-pass-encoder':
                    assert(this.renderPassEncoder !== undefined);
                    this.renderPassEncoder.setPipeline(renderPipeline);
                    this.renderPassEncoder.setBindGroup(0, bindGroup);
                    this.renderPassEncoder.draw(6);
                    break;
                }
                break;
              }
            case 'compute-pass-encoder':{
                const module = this.device.createShaderModule({
                  code: `
                @group(0) @binding(0) var outputTex: texture_storage_2d<rgba8unorm, write>;

                @compute @workgroup_size(8, 8)
                fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
                  if (any(gid.xy >= vec2<u32>(textureDimensions(outputTex)))) {
                    return;
                  }
                  let coord = vec2<i32>(gid.xy);
                  textureStore(outputTex, coord, ${storedValue});
                }
              `
                });
                const computePipeline = this.device.createComputePipeline({
                  layout: this.device.createPipelineLayout({
                    bindGroupLayouts: [bindGroupLayout]
                  }),
                  compute: {
                    module,
                    entryPoint: 'main'
                  }
                });

                assert(this.computePassEncoder !== undefined);
                this.computePassEncoder.setPipeline(computePipeline);
                this.computePassEncoder.setBindGroup(0, bindGroup);
                this.computePassEncoder.dispatchWorkgroups(
                  Math.ceil(this.kTextureSize[0] / 8),
                  Math.ceil(this.kTextureSize[1] / 8)
                );
                break;
              }
            default:
              unreachable();
          }
          break;
        }
      case 't2b-copy':
      case 'sample':
        unreachable();
    }
  }
}

g.test('rw').
desc(
  `
    Perform a 'read' operations on a texture subresource, followed by a 'write' operation.
    Operations are separated by a 'boundary' (pass, encoder, queue-op, etc.).
    Test that the results are synchronized.
    The read should not see the contents written by the subsequent write.`
).
params((u) =>
u.
combine('boundary', kOperationBoundaries).
expand('_context', (p) => kBoundaryInfo[p.boundary].contexts).
expandWithParams(function* ({ _context }) {
  for (const read of kAllReadOps) {
    for (const write of kAllWriteOps) {
      if (checkOpsValidForContext([read, write], _context)) {
        yield {
          read: { op: read, in: _context[0] },
          write: { op: write, in: _context[1] }
        };
      }
    }
  }
})
).
fn((t) => {
  const helper = new TextureSyncTestHelper(t, {
    usage:
    GPUTextureUsage.COPY_DST |
    kOpInfo[t.params.read.op].readUsage |
    kOpInfo[t.params.write.op].writeUsage
  });
  // [2] Use non-solid-color texture value.
  const texelValue1 = { R: 0, G: 1, B: 0, A: 1 };
  const texelValue2 = { R: 1, G: 0, B: 0, A: 1 };

  // Initialize the texture with something.
  helper.performWriteOp({ op: 'write-texture', in: 'queue' }, texelValue1);
  const readbackTexture = helper.performReadOp(t.params.read);
  helper.ensureBoundary(t.params.boundary);
  helper.performWriteOp(t.params.write, texelValue2);
  helper.ensureSubmit();

  // Contents should be the first value written, not the second.
  t.expectSingleColor(readbackTexture, helper.kTextureFormat, {
    size: [...helper.kTextureSize, 1],
    exp: texelValue1
  });
});

g.test('wr').
desc(
  `
    Perform a 'write' operation on a texture subresource, followed by a 'read' operation.
    Operations are separated by a 'boundary' (pass, encoder, queue-op, etc.).
    Test that the results are synchronized.
    The read should see exactly the contents written by the previous write.

    - TODO: Use non-solid-color texture contents [2]`
).
params((u) =>
u.
combine('boundary', kOperationBoundaries).
expand('_context', (p) => kBoundaryInfo[p.boundary].contexts).
expandWithParams(function* ({ _context }) {
  for (const read of kAllReadOps) {
    for (const write of kAllWriteOps) {
      if (checkOpsValidForContext([write, read], _context)) {
        yield {
          write: { op: write, in: _context[0] },
          read: { op: read, in: _context[1] }
        };
      }
    }
  }
})
).
fn((t) => {
  const helper = new TextureSyncTestHelper(t, {
    usage: kOpInfo[t.params.read.op].readUsage | kOpInfo[t.params.write.op].writeUsage
  });
  // [2] Use non-solid-color texture value.
  const texelValue = { R: 0, G: 1, B: 0, A: 1 };

  helper.performWriteOp(t.params.write, texelValue);
  helper.ensureBoundary(t.params.boundary);
  const readbackTexture = helper.performReadOp(t.params.read);
  helper.ensureSubmit();

  // Contents should be exactly the values written.
  t.expectSingleColor(readbackTexture, helper.kTextureFormat, {
    size: [...helper.kTextureSize, 1],
    exp: texelValue
  });
});

g.test('ww').
desc(
  `
    Perform a 'first' write operation on a texture subresource, followed by a 'second' write operation.
    Operations are separated by a 'boundary' (pass, encoder, queue-op, etc.).
    Test that the results are synchronized.
    The second write should overwrite the contents of the first.`
).
params((u) =>
u.
combine('boundary', kOperationBoundaries).
expand('_context', (p) => kBoundaryInfo[p.boundary].contexts).
expandWithParams(function* ({ _context }) {
  for (const first of kAllWriteOps) {
    for (const second of kAllWriteOps) {
      if (checkOpsValidForContext([first, second], _context)) {
        yield {
          first: { op: first, in: _context[0] },
          second: { op: second, in: _context[1] }
        };
      }
    }
  }
})
).
fn((t) => {
  const helper = new TextureSyncTestHelper(t, {
    usage:
    GPUTextureUsage.COPY_SRC |
    kOpInfo[t.params.first.op].writeUsage |
    kOpInfo[t.params.second.op].writeUsage
  });
  // [2] Use non-solid-color texture value.
  const texelValue1 = { R: 1, G: 0, B: 0, A: 1 };
  const texelValue2 = { R: 0, G: 1, B: 0, A: 1 };

  helper.performWriteOp(t.params.first, texelValue1);
  helper.ensureBoundary(t.params.boundary);
  helper.performWriteOp(t.params.second, texelValue2);
  helper.ensureSubmit();

  // Read back the contents so we can test the result.
  const readbackTexture = helper.performReadOp({ op: 't2t-copy', in: 'command-encoder' });
  helper.ensureSubmit();

  // Contents should be the second value written.
  t.expectSingleColor(readbackTexture, helper.kTextureFormat, {
    size: [...helper.kTextureSize, 1],
    exp: texelValue2
  });
});

g.test('rw,single_pass,load_store').
desc(
  `
    TODO: Test memory synchronization when loading from a texture subresource in a single pass and storing to it.`
).
unimplemented();

g.test('rw,single_pass,load_resolve').
desc(
  `
    TODO: Test memory synchronization when loading from a texture subresource in a single pass and resolving to it.`
).
unimplemented();