/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert, unreachable } from '../../../common/util/util.js';
import { kTextureFormatInfo } from '../../format_info.js';
import { gammaDecompress, float32ToFloat16Bits } from '../../util/conversion.js';
import { align } from '../../util/math.js';

import { runRefTest } from './gpu_ref_test.js';

export function run(format, targets) {
  runRefTest(async t => {
    let shaderValue = 0x66 / 0xff;
    let isOutputSrgb = false;
    switch (format) {
      case 'bgra8unorm':
      case 'rgba8unorm':
      case 'rgba16float':
        break;
      case 'bgra8unorm-srgb':
      case 'rgba8unorm-srgb':
        // NOTE: "-srgb" cases haven't been tested (there aren't any .html files that use them).

        // Reverse gammaCompress to get same value shader output as non-srgb formats:
        shaderValue = gammaDecompress(shaderValue);
        isOutputSrgb = true;
        break;
      default:
        unreachable();
    }

    const shaderValueStr = shaderValue.toFixed(5);

    function copyBufferToTexture(ctx) {
      const rows = ctx.canvas.height;
      const bytesPerPixel = kTextureFormatInfo[format].color.bytes;
      if (bytesPerPixel === undefined) {
        unreachable();
      }
      const bytesPerRow = align(bytesPerPixel * ctx.canvas.width, 256);
      const componentsPerPixel = 4;

      const buffer = t.device.createBuffer({
        mappedAtCreation: true,
        size: rows * bytesPerRow,
        usage: GPUBufferUsage.COPY_SRC,
      });
      let red;
      let green;
      let blue;
      let yellow;

      const mapping = buffer.getMappedRange();
      let data;
      switch (format) {
        case 'bgra8unorm':
        case 'bgra8unorm-srgb':
          {
            data = new Uint8Array(mapping);
            red = new Uint8Array([0x00, 0x00, 0x66, 0xff]);
            green = new Uint8Array([0x00, 0x66, 0x00, 0xff]);
            blue = new Uint8Array([0x66, 0x00, 0x00, 0xff]);
            yellow = new Uint8Array([0x00, 0x66, 0x66, 0xff]);
          }
          break;
        case 'rgba8unorm':
        case 'rgba8unorm-srgb':
          {
            data = new Uint8Array(mapping);
            red = new Uint8Array([0x66, 0x00, 0x00, 0xff]);
            green = new Uint8Array([0x00, 0x66, 0x00, 0xff]);
            blue = new Uint8Array([0x00, 0x00, 0x66, 0xff]);
            yellow = new Uint8Array([0x66, 0x66, 0x00, 0xff]);
          }
          break;
        case 'rgba16float':
          {
            data = new Uint16Array(mapping);
            red = new Uint16Array([
              float32ToFloat16Bits(0.4),
              float32ToFloat16Bits(0.0),
              float32ToFloat16Bits(0.0),
              float32ToFloat16Bits(1.0),
            ]);

            green = new Uint16Array([
              float32ToFloat16Bits(0.0),
              float32ToFloat16Bits(0.4),
              float32ToFloat16Bits(0.0),
              float32ToFloat16Bits(1.0),
            ]);

            blue = new Uint16Array([
              float32ToFloat16Bits(0.0),
              float32ToFloat16Bits(0.0),
              float32ToFloat16Bits(0.4),
              float32ToFloat16Bits(1.0),
            ]);

            yellow = new Uint16Array([
              float32ToFloat16Bits(0.4),
              float32ToFloat16Bits(0.4),
              float32ToFloat16Bits(0.0),
              float32ToFloat16Bits(1.0),
            ]);
          }
          break;
        default:
          unreachable();
      }

      for (let i = 0; i < ctx.canvas.width; ++i)
        for (let j = 0; j < ctx.canvas.height; ++j) {
          let pixel;
          if (i < ctx.canvas.width / 2) {
            if (j < ctx.canvas.height / 2) {
              pixel = red;
            } else {
              pixel = blue;
            }
          } else {
            if (j < ctx.canvas.height / 2) {
              pixel = green;
            } else {
              pixel = yellow;
            }
          }
          data.set(pixel, (i + j * (bytesPerRow / bytesPerPixel)) * componentsPerPixel);
        }
      buffer.unmap();

      const encoder = t.device.createCommandEncoder();
      encoder.copyBufferToTexture({ buffer, bytesPerRow }, { texture: ctx.getCurrentTexture() }, [
        ctx.canvas.width,
        ctx.canvas.height,
        1,
      ]);

      t.device.queue.submit([encoder.finish()]);
    }

    function getImageBitmap(ctx) {
      const data = new Uint8ClampedArray(ctx.canvas.width * ctx.canvas.height * 4);
      for (let i = 0; i < ctx.canvas.width; ++i)
        for (let j = 0; j < ctx.canvas.height; ++j) {
          const offset = (i + j * ctx.canvas.width) * 4;
          if (i < ctx.canvas.width / 2) {
            if (j < ctx.canvas.height / 2) {
              data.set([0x66, 0x00, 0x00, 0xff], offset);
            } else {
              data.set([0x00, 0x00, 0x66, 0xff], offset);
            }
          } else {
            if (j < ctx.canvas.height / 2) {
              data.set([0x00, 0x66, 0x00, 0xff], offset);
            } else {
              data.set([0x66, 0x66, 0x00, 0xff], offset);
            }
          }
        }
      const imageData = new ImageData(data, ctx.canvas.width, ctx.canvas.height);
      return createImageBitmap(imageData);
    }

    function setupSrcTexture(imageBitmap) {
      const [srcWidth, srcHeight] = [imageBitmap.width, imageBitmap.height];
      const srcTexture = t.device.createTexture({
        size: [srcWidth, srcHeight, 1],
        format,
        usage:
          GPUTextureUsage.TEXTURE_BINDING |
          GPUTextureUsage.RENDER_ATTACHMENT |
          GPUTextureUsage.COPY_DST |
          GPUTextureUsage.COPY_SRC,
      });
      t.device.queue.copyExternalImageToTexture({ source: imageBitmap }, { texture: srcTexture }, [
        imageBitmap.width,
        imageBitmap.height,
      ]);

      return srcTexture;
    }

    async function copyExternalImageToTexture(ctx) {
      const imageBitmap = await getImageBitmap(ctx);
      t.device.queue.copyExternalImageToTexture(
        { source: imageBitmap },
        { texture: ctx.getCurrentTexture() },
        [imageBitmap.width, imageBitmap.height]
      );
    }

    async function copyTextureToTexture(ctx) {
      const imageBitmap = await getImageBitmap(ctx);
      const srcTexture = setupSrcTexture(imageBitmap);

      const encoder = t.device.createCommandEncoder();
      encoder.copyTextureToTexture(
        { texture: srcTexture, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
        { texture: ctx.getCurrentTexture(), mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
        [imageBitmap.width, imageBitmap.height, 1]
      );

      t.device.queue.submit([encoder.finish()]);
    }

    async function DrawTextureSample(ctx) {
      const imageBitmap = await getImageBitmap(ctx);
      const srcTexture = setupSrcTexture(imageBitmap);

      const pipeline = t.device.createRenderPipeline({
        layout: 'auto',
        vertex: {
          module: t.device.createShaderModule({
            code: `
struct VertexOutput {
  @builtin(position) Position : vec4<f32>,
  @location(0) fragUV : vec2<f32>,
}

@vertex
fn main(@builtin(vertex_index) VertexIndex : u32) -> VertexOutput {
  var pos = array<vec2<f32>, 6>(
      vec2<f32>( 1.0,  1.0),
      vec2<f32>( 1.0, -1.0),
      vec2<f32>(-1.0, -1.0),
      vec2<f32>( 1.0,  1.0),
      vec2<f32>(-1.0, -1.0),
      vec2<f32>(-1.0,  1.0));

  var uv = array<vec2<f32>, 6>(
      vec2<f32>(1.0, 0.0),
      vec2<f32>(1.0, 1.0),
      vec2<f32>(0.0, 1.0),
      vec2<f32>(1.0, 0.0),
      vec2<f32>(0.0, 1.0),
      vec2<f32>(0.0, 0.0));

  var output : VertexOutput;
  output.Position = vec4<f32>(pos[VertexIndex], 0.0, 1.0);
  output.fragUV = uv[VertexIndex];
  return output;
}
            `,
          }),
          entryPoint: 'main',
        },
        fragment: {
          module: t.device.createShaderModule({
            // NOTE: "-srgb" cases haven't been tested (there aren't any .html files that use them).
            code: `
@group(0) @binding(0) var mySampler: sampler;
@group(0) @binding(1) var myTexture: texture_2d<f32>;

fn gammaDecompress(n: f32) -> f32 {
  var r = n;
  if (r <= 0.04045) {
    r = r * 25.0 / 323.0;
  } else {
    r = pow((200.0 * r + 11.0) / 121.0, 12.0 / 5.0);
  }
  r = clamp(r, 0.0, 1.0);
  return r;
}

@fragment
fn srgbMain(@location(0) fragUV: vec2<f32>) -> @location(0) vec4<f32> {
  var result = textureSample(myTexture, mySampler, fragUV);
  result.r = gammaDecompress(result.r);
  result.g = gammaDecompress(result.g);
  result.b = gammaDecompress(result.b);
  return result;
}

@fragment
fn linearMain(@location(0) fragUV: vec2<f32>) -> @location(0) vec4<f32> {
  return textureSample(myTexture, mySampler, fragUV);
}
            `,
          }),
          entryPoint: isOutputSrgb ? 'srgbMain' : 'linearMain',
          targets: [{ format }],
        },
        primitive: {
          topology: 'triangle-list',
        },
      });

      const sampler = t.device.createSampler({
        magFilter: 'nearest',
        minFilter: 'nearest',
      });

      const uniformBindGroup = t.device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
          {
            binding: 0,
            resource: sampler,
          },
          {
            binding: 1,
            resource: srcTexture.createView(),
          },
        ],
      });

      const renderPassDescriptor = {
        colorAttachments: [
          {
            view: ctx.getCurrentTexture().createView(),

            clearValue: { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
            loadOp: 'clear',
            storeOp: 'store',
          },
        ],
      };

      const commandEncoder = t.device.createCommandEncoder();
      const passEncoder = commandEncoder.beginRenderPass(renderPassDescriptor);
      passEncoder.setPipeline(pipeline);
      passEncoder.setBindGroup(0, uniformBindGroup);
      passEncoder.draw(6, 1, 0, 0);
      passEncoder.end();
      t.device.queue.submit([commandEncoder.finish()]);
    }

    function DrawVertexColor(ctx) {
      const pipeline = t.device.createRenderPipeline({
        layout: 'auto',
        vertex: {
          module: t.device.createShaderModule({
            code: `
struct VertexOutput {
  @builtin(position) Position : vec4<f32>,
  @location(0) fragColor : vec4<f32>,
}

@vertex
fn main(@builtin(vertex_index) VertexIndex : u32) -> VertexOutput {
  var pos = array<vec2<f32>, 6>(
      vec2<f32>( 0.5,  0.5),
      vec2<f32>( 0.5, -0.5),
      vec2<f32>(-0.5, -0.5),
      vec2<f32>( 0.5,  0.5),
      vec2<f32>(-0.5, -0.5),
      vec2<f32>(-0.5,  0.5));

  var offset = array<vec2<f32>, 4>(
    vec2<f32>( -0.5,  0.5),
    vec2<f32>( 0.5, 0.5),
    vec2<f32>(-0.5, -0.5),
    vec2<f32>( 0.5,  -0.5));

  var color = array<vec4<f32>, 4>(
      vec4<f32>(${shaderValueStr}, 0.0, 0.0, 1.0),
      vec4<f32>(0.0, ${shaderValueStr}, 0.0, 1.0),
      vec4<f32>(0.0, 0.0, ${shaderValueStr}, 1.0),
      vec4<f32>(${shaderValueStr}, ${shaderValueStr}, 0.0, 1.0));

  var output : VertexOutput;
  output.Position = vec4<f32>(pos[VertexIndex % 6u] + offset[VertexIndex / 6u], 0.0, 1.0);
  output.fragColor = color[VertexIndex / 6u];
  return output;
}
            `,
          }),
          entryPoint: 'main',
        },
        fragment: {
          module: t.device.createShaderModule({
            code: `
@fragment
fn main(@location(0) fragColor: vec4<f32>) -> @location(0) vec4<f32> {
  return fragColor;
}
            `,
          }),
          entryPoint: 'main',
          targets: [{ format }],
        },
        primitive: {
          topology: 'triangle-list',
        },
      });

      const renderPassDescriptor = {
        colorAttachments: [
          {
            view: ctx.getCurrentTexture().createView(),

            clearValue: { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
            loadOp: 'clear',
            storeOp: 'store',
          },
        ],
      };

      const commandEncoder = t.device.createCommandEncoder();
      const passEncoder = commandEncoder.beginRenderPass(renderPassDescriptor);
      passEncoder.setPipeline(pipeline);
      passEncoder.draw(24, 1, 0, 0);
      passEncoder.end();
      t.device.queue.submit([commandEncoder.finish()]);
    }

    function DrawFragcoord(ctx) {
      const halfCanvasWidthStr = (ctx.canvas.width / 2).toFixed();
      const halfCanvasHeightStr = (ctx.canvas.height / 2).toFixed();
      const pipeline = t.device.createRenderPipeline({
        layout: 'auto',
        vertex: {
          module: t.device.createShaderModule({
            code: `
struct VertexOutput {
  @builtin(position) Position : vec4<f32>
}

@vertex
fn main(@builtin(vertex_index) VertexIndex : u32) -> VertexOutput {
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
            `,
          }),
          entryPoint: 'main',
        },
        fragment: {
          module: t.device.createShaderModule({
            code: `
@group(0) @binding(0) var mySampler: sampler;
@group(0) @binding(1) var myTexture: texture_2d<f32>;

@fragment
fn main(@builtin(position) fragcoord: vec4<f32>) -> @location(0) vec4<f32> {
  var coord = vec2<u32>(floor(fragcoord.xy));
  var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  if (coord.x < ${halfCanvasWidthStr}u) {
    if (coord.y < ${halfCanvasHeightStr}u) {
      color.r = ${shaderValueStr};
    } else {
      color.b = ${shaderValueStr};
    }
  } else {
    if (coord.y < ${halfCanvasHeightStr}u) {
      color.g = ${shaderValueStr};
    } else {
      color.r = ${shaderValueStr};
      color.g = ${shaderValueStr};
    }
  }
  return color;
}
            `,
          }),
          entryPoint: 'main',
          targets: [{ format }],
        },
        primitive: {
          topology: 'triangle-list',
        },
      });

      const renderPassDescriptor = {
        colorAttachments: [
          {
            view: ctx.getCurrentTexture().createView(),

            clearValue: { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
            loadOp: 'clear',
            storeOp: 'store',
          },
        ],
      };

      const commandEncoder = t.device.createCommandEncoder();
      const passEncoder = commandEncoder.beginRenderPass(renderPassDescriptor);
      passEncoder.setPipeline(pipeline);
      passEncoder.draw(6, 1, 0, 0);
      passEncoder.end();
      t.device.queue.submit([commandEncoder.finish()]);
    }

    function FragmentTextureStore(ctx) {
      const halfCanvasWidthStr = (ctx.canvas.width / 2).toFixed();
      const halfCanvasHeightStr = (ctx.canvas.height / 2).toFixed();
      const pipeline = t.device.createRenderPipeline({
        layout: 'auto',
        vertex: {
          module: t.device.createShaderModule({
            code: `
struct VertexOutput {
  @builtin(position) Position : vec4<f32>
}

@vertex
fn main(@builtin(vertex_index) VertexIndex : u32) -> VertexOutput {
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
            `,
          }),
          entryPoint: 'main',
        },
        fragment: {
          module: t.device.createShaderModule({
            code: `
@group(0) @binding(0) var outImage : texture_storage_2d<${format}, write>;

@fragment
fn main(@builtin(position) fragcoord: vec4<f32>) -> @location(0) vec4<f32> {
  var coord = vec2<u32>(floor(fragcoord.xy));
  var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  if (coord.x < ${halfCanvasWidthStr}u) {
    if (coord.y < ${halfCanvasHeightStr}u) {
      color.r = ${shaderValueStr};
    } else {
      color.b = ${shaderValueStr};
    }
  } else {
    if (coord.y < ${halfCanvasHeightStr}u) {
      color.g = ${shaderValueStr};
    } else {
      color.r = ${shaderValueStr};
      color.g = ${shaderValueStr};
    }
  }
  textureStore(outImage, vec2<i32>(coord), color);
  return color;
}
            `,
          }),
          entryPoint: 'main',
          targets: [{ format }],
        },
        primitive: {
          topology: 'triangle-list',
        },
      });

      const bg = t.device.createBindGroup({
        entries: [{ binding: 0, resource: ctx.getCurrentTexture().createView() }],
        layout: pipeline.getBindGroupLayout(0),
      });

      const outputTexture = t.device.createTexture({
        format,
        size: [ctx.canvas.width, ctx.canvas.height, 1],
        usage: GPUTextureUsage.RENDER_ATTACHMENT,
      });

      const renderPassDescriptor = {
        colorAttachments: [
          {
            view: outputTexture.createView(),

            clearValue: { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
            loadOp: 'clear',
            storeOp: 'store',
          },
        ],
      };

      const commandEncoder = t.device.createCommandEncoder();
      const passEncoder = commandEncoder.beginRenderPass(renderPassDescriptor);
      passEncoder.setPipeline(pipeline);
      passEncoder.setBindGroup(0, bg);
      passEncoder.draw(6, 1, 0, 0);
      passEncoder.end();
      t.device.queue.submit([commandEncoder.finish()]);
    }

    function ComputeWorkgroup1x1TextureStore(ctx) {
      const halfCanvasWidthStr = (ctx.canvas.width / 2).toFixed();
      const halfCanvasHeightStr = (ctx.canvas.height / 2).toFixed();
      const pipeline = t.device.createComputePipeline({
        layout: 'auto',
        compute: {
          module: t.device.createShaderModule({
            code: `
@group(0) @binding(0) var outImage : texture_storage_2d<${format}, write>;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) GlobalInvocationID : vec3<u32>) {
  var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  if (GlobalInvocationID.x < ${halfCanvasWidthStr}u) {
    if (GlobalInvocationID.y < ${halfCanvasHeightStr}u) {
      color.r = ${shaderValueStr};
    } else {
      color.b = ${shaderValueStr};
    }
  } else {
    if (GlobalInvocationID.y < ${halfCanvasHeightStr}u) {
      color.g = ${shaderValueStr};
    } else {
      color.r = ${shaderValueStr};
      color.g = ${shaderValueStr};
    }
  }
  textureStore(outImage, vec2<i32>(GlobalInvocationID.xy), color);
  return;
}
          `,
          }),
          entryPoint: 'main',
        },
      });

      const bg = t.device.createBindGroup({
        entries: [{ binding: 0, resource: ctx.getCurrentTexture().createView() }],
        layout: pipeline.getBindGroupLayout(0),
      });

      const encoder = t.device.createCommandEncoder();
      const pass = encoder.beginComputePass();
      pass.setPipeline(pipeline);
      pass.setBindGroup(0, bg);
      pass.dispatchWorkgroups(ctx.canvas.width, ctx.canvas.height, 1);
      pass.end();
      t.device.queue.submit([encoder.finish()]);
    }

    function ComputeWorkgroup16x16TextureStore(ctx) {
      const canvasWidthStr = ctx.canvas.width.toFixed();
      const canvasHeightStr = ctx.canvas.height.toFixed();
      const halfCanvasWidthStr = (ctx.canvas.width / 2).toFixed();
      const halfCanvasHeightStr = (ctx.canvas.height / 2).toFixed();
      const pipeline = t.device.createComputePipeline({
        layout: 'auto',
        compute: {
          module: t.device.createShaderModule({
            code: `
@group(0) @binding(0) var outImage : texture_storage_2d<${format}, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) GlobalInvocationID : vec3<u32>) {
  if (GlobalInvocationID.x >= ${canvasWidthStr}u ||
      GlobalInvocationID.y >= ${canvasHeightStr}u) {
        return;
  }
  var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  if (GlobalInvocationID.x < ${halfCanvasWidthStr}u) {
    if (GlobalInvocationID.y < ${halfCanvasHeightStr}u) {
      color.r = ${shaderValueStr};
    } else {
      color.b = ${shaderValueStr};
    }
  } else {
    if (GlobalInvocationID.y < ${halfCanvasHeightStr}u) {
      color.g = ${shaderValueStr};
    } else {
      color.r = ${shaderValueStr};
      color.g = ${shaderValueStr};
    }
  }
  textureStore(outImage, vec2<i32>(GlobalInvocationID.xy), color);
  return;
}
            `,
          }),
          entryPoint: 'main',
        },
      });

      const bg = t.device.createBindGroup({
        entries: [{ binding: 0, resource: ctx.getCurrentTexture().createView() }],
        layout: pipeline.getBindGroupLayout(0),
      });

      const encoder = t.device.createCommandEncoder();
      const pass = encoder.beginComputePass();
      pass.setPipeline(pipeline);
      pass.setBindGroup(0, bg);
      pass.dispatchWorkgroups(
        align(ctx.canvas.width, 16) / 16,
        align(ctx.canvas.height, 16) / 16,
        1
      );

      pass.end();
      t.device.queue.submit([encoder.finish()]);
    }

    for (const { cvs, writeCanvasMethod } of targets) {
      const ctx = cvs.getContext('webgpu');
      assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

      let usage;
      switch (writeCanvasMethod) {
        case 'copyBufferToTexture':
        case 'copyTextureToTexture':
          usage = GPUTextureUsage.COPY_DST;
          break;
        case 'copyExternalImageToTexture':
          usage = GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT;
          break;
        case 'DrawTextureSample':
        case 'DrawVertexColor':
        case 'DrawFragcoord':
          usage = GPUTextureUsage.RENDER_ATTACHMENT;
          break;
        case 'FragmentTextureStore':
        case 'ComputeWorkgroup1x1TextureStore':
        case 'ComputeWorkgroup16x16TextureStore':
          usage = GPUTextureUsage.STORAGE_BINDING;
          break;
        default:
          unreachable();
      }

      ctx.configure({
        device: t.device,
        format,
        usage,
      });

      switch (writeCanvasMethod) {
        case 'copyBufferToTexture':
          copyBufferToTexture(ctx);
          break;
        case 'copyExternalImageToTexture':
          await copyExternalImageToTexture(ctx);
          break;
        case 'copyTextureToTexture':
          await copyTextureToTexture(ctx);
          break;
        case 'DrawTextureSample':
          await DrawTextureSample(ctx);
          break;
        case 'DrawVertexColor':
          DrawVertexColor(ctx);
          break;
        case 'DrawFragcoord':
          DrawFragcoord(ctx);
          break;
        case 'FragmentTextureStore':
          FragmentTextureStore(ctx);
          break;
        case 'ComputeWorkgroup1x1TextureStore':
          ComputeWorkgroup1x1TextureStore(ctx);
          break;
        case 'ComputeWorkgroup16x16TextureStore':
          ComputeWorkgroup16x16TextureStore(ctx);
          break;
        default:
          unreachable();
      }
    }
  });
}
