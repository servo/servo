/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test that with the limits set to their maximum we can actually use
the maximum number of storage buffers, storage textures, and fragment outputs
at the same time.

In particular, OpenGL ES 3.1 has GL_MAX_COMBINED_SHADER_OUTPUT_RESOURCES which
the spec says is the combination of storage textures, storage buffers, and
fragment shader outputs. This test checks that the whatever values the WebGPU
implementation allows, all of them are useable.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { range } from '../../../../common/util/util.js';
import { getColorRenderByteCost } from '../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';
import * as ttu from '../../../texture_test_utils.js';
import { TexelView } from '../../../util/texture/texel_view.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('max_storage_buffer_texture_frag_outputs').
desc(
  `
    Use the maximum number of storage buffer, storage texture, and fragment stage outputs
    `
).
params((u) => u.combine('format', ['r8uint', 'rgba8uint', 'rgba32uint'])).
fn((t) => {
  const { format } = t.params;
  const { device } = t;

  const kWidth = 4;
  const kHeight = 4;

  const numColorAttachments = Math.min(
    device.limits.maxColorAttachments,
    device.limits.maxColorAttachmentBytesPerSample / getColorRenderByteCost(format)
  );
  const numStorageBuffers =
  device.limits.maxStorageBuffersInFragmentStage ??
  device.limits.maxStorageBuffersPerShaderStage;
  const numStorageTextures =
  device.limits.maxStorageTexturesInFragmentStage ??
  device.limits.maxStorageTexturesPerShaderStage;

  const code = `
${range(
    numStorageBuffers,
    (i) => `@group(0) @binding(${i}) var<storage, read_write> sb${i}: array<vec4u>;`
  ).join('\n')}

${range(
    numStorageTextures,
    (i) => `@group(1) @binding(${i}) var st${i}: texture_storage_2d<rgba32uint, write>;`
  ).join('\n')}

struct FragOut {
${range(numColorAttachments, (i) => `  @location(${i}) f${i}: vec4u,`).join('\n')}
};

@vertex fn vs(@builtin(vertex_index) vNdx: u32) -> @builtin(position) vec4f {
  let pos = array(
    vec2f(-1,  3),
    vec2f( 3, -1),
    vec2f(-1, -1),
  );
  return vec4f(pos[vNdx], 0, 1);
}

@fragment fn fs(@builtin(position) position: vec4f) -> FragOut {
  let p = vec4u(position);
  let ndx = p.y * ${kWidth} + p.x;

${range(numStorageBuffers, (i) => `  sb${i}[ndx] = p + ${i};`).join('\n')}

${range(numStorageTextures, (i) => `  textureStore(st${i}, p.xy, p + ${i} * 2);`).join('\n')}

  var fragOut: FragOut;
${range(numColorAttachments, (i) => `  fragOut.f${i} = vec4u(p + ${i} * 3);`).join('\n')}
  return fragOut;
}
    `;

  t.debug(code);
  const module = device.createShaderModule({ code });
  const pipeline = device.createRenderPipeline({
    layout: 'auto',
    vertex: { module },
    fragment: {
      module,
      targets: range(numColorAttachments, (i) => ({ format }))
    },
    depthStencil: {
      depthWriteEnabled: true,
      depthCompare: 'less',
      format: 'depth24plus-stencil8'
    }
  });

  const size = kWidth * kHeight * 4 * 4;
  const usage = GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC;
  const storageBuffers = range(numStorageBuffers, (i) => t.createBufferTracked({ size, usage }));

  const storageTextures = range(numStorageTextures, (i) =>
  t.createTextureTracked({
    format: 'rgba32uint',
    size: [kWidth, kHeight],
    usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.COPY_SRC
  })
  );

  const targets = range(numColorAttachments, (i) =>
  t.createTextureTracked({
    format,
    size: [kWidth, kHeight],
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  })
  );

  const bindGroup0 = device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: storageBuffers.map((buffer, i) => ({ binding: i, resource: { buffer } }))
  });

  const bindGroup1 = device.createBindGroup({
    layout: pipeline.getBindGroupLayout(1),
    entries: storageTextures.map((storageTexture, i) => ({
      binding: i,
      resource: storageTexture.createView()
    }))
  });

  // Note: the depth-stencil attachment is just to add more output.
  // We do not check its contents.
  const encoder = device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: targets.map((texture) => ({
      loadOp: 'clear',
      storeOp: 'store',
      view: texture.createView()
    })),
    depthStencilAttachment: {
      view: t.
      createTextureTracked({
        format: 'depth24plus-stencil8',
        usage: GPUTextureUsage.RENDER_ATTACHMENT,
        size: [kWidth, kHeight]
      }).
      createView(),
      depthClearValue: 1.0,
      depthLoadOp: 'clear',
      depthStoreOp: 'store',
      stencilClearValue: 0,
      stencilLoadOp: 'clear',
      stencilStoreOp: 'store'
    }
  });
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup0);
  pass.setBindGroup(1, bindGroup1);
  pass.draw(3);
  pass.end();
  device.queue.submit([encoder.finish()]);

  const fillExpected = (expected, i) => {
    for (let y = 0; y < kHeight; ++y) {
      for (let x = 0; x < kWidth; ++x) {
        const off = (y * kWidth + x) * 4;
        expected[off + 0] = x + i;
        expected[off + 1] = y + i;
        expected[off + 2] = i;
        expected[off + 3] = 1 + i;
      }
    }
    return expected;
  };

  const makeExpectedRGBA32Uint = (i) => {
    const expected = new Uint32Array(size / 4);
    return fillExpected(expected, i);
  };

  const makeExpectedRGBA8Uint = (i) => {
    const expected = new Uint8Array(kWidth * kHeight * 4);
    return fillExpected(expected, i);
  };

  const makeExpectedR8Uint = (i) => {
    const temp = makeExpectedRGBA8Uint(i);
    const expected = new Uint8Array(kWidth * kHeight);
    for (let i = 0; i < expected.length; ++i) {
      expected[i] = temp[i * 4];
    }
    return expected;
  };

  storageBuffers.forEach((buffer, i) => {
    t.expectGPUBufferValuesEqual(buffer, makeExpectedRGBA32Uint(i));
  });

  storageTextures.forEach((texture, i) => {
    ttu.expectTexelViewComparisonIsOkInTexture(
      t,
      { texture },
      TexelView.fromTextureDataByReference(
        'rgba32uint',
        new Uint8Array(makeExpectedRGBA32Uint(i * 2).buffer),
        {
          bytesPerRow: kWidth * 16,
          rowsPerImage: kHeight,
          subrectOrigin: [0, 0],
          subrectSize: [kWidth, kHeight]
        }
      ),
      [kWidth, kHeight]
    );
  });

  targets.forEach((texture, i) => {
    let expected;
    let bytesPerRow;
    switch (format) {
      case 'r8uint':
        expected = makeExpectedR8Uint(i * 3);
        bytesPerRow = kWidth;
        break;
      case 'rgba8uint':
        expected = makeExpectedRGBA8Uint(i * 3);
        bytesPerRow = kWidth * 4;
        break;
      case 'rgba32uint':
        expected = new Uint8Array(makeExpectedRGBA32Uint(i * 3).buffer);
        bytesPerRow = kWidth * 16;
        break;
    }
    ttu.expectTexelViewComparisonIsOkInTexture(
      t,
      { texture },
      TexelView.fromTextureDataByReference(format, expected, {
        bytesPerRow,
        rowsPerImage: kHeight,
        subrectOrigin: [0, 0],
        subrectSize: [kWidth, kHeight]
      }),
      [kWidth, kHeight]
    );
  });
});