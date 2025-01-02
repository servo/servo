/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test rendering to 3d texture slices.
- Render to same slice on different render pass, different textures, or texture [1, 1, N]'s different mip levels
- Render to different slices at mip levels on same texture in render pass
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kTextureFormatInfo } from '../../../format_info.js';
import { GPUTest } from '../../../gpu_test.js';
import { kBytesPerRowAlignment } from '../../../util/texture/layout.js';

const kSize = 4;
const kFormat = 'rgba8unorm';

class F extends GPUTest {
  createShaderModule(attachmentCount = 1) {
    let locations = '';
    let outputs = '';
    for (let i = 0; i < attachmentCount; i++) {
      locations = locations + `@location(${i}) color${i} : vec4f, \n`;
      outputs = outputs + `output.color${i} = vec4f(0.0, 1.0, 0.0, 1.0);\n`;
    }

    return this.device.createShaderModule({
      code: `
        struct Output {
          ${locations}
        }

        @vertex
        fn main_vs(@builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4<f32> {
          var pos : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
              // Triangle is slightly extended so its edge doesn't cut through pixel centers.
              vec2<f32>(-1.0, 1.01),
              vec2<f32>(1.01, -1.0),
              vec2<f32>(-1.0, -1.0));
          return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
        }

        @fragment
        fn main_fs() -> Output {
          var output : Output;
          ${outputs}
          return output;
        }
        `
    });
  }

  getBufferSizeAndOffset(
  attachmentWidth,
  attachmentHeight,
  attachmentCount)
  {
    const bufferSize =
    (attachmentCount * attachmentHeight - 1) * kBytesPerRowAlignment + attachmentWidth * 4;
    const bufferOffset = attachmentCount > 1 ? attachmentHeight * kBytesPerRowAlignment : 0;
    return { bufferSize, bufferOffset };
  }

  checkAttachmentResult(
  attachmentWidth,
  attachmentHeight,
  attachmentCount,
  buffer)
  {
    const { bufferSize, bufferOffset } = this.getBufferSizeAndOffset(
      attachmentWidth,
      attachmentHeight,
      attachmentCount
    );
    const expectedData = new Uint8Array(bufferSize);
    for (let i = 0; i < attachmentCount; i++) {
      for (let j = 0; j < attachmentHeight; j++) {
        for (let k = 0; k < attachmentWidth; k++) {
          expectedData[i * bufferOffset + j * 256 + k * 4] = k <= j ? 0x00 : 0xff;
          expectedData[i * bufferOffset + j * 256 + k * 4 + 1] = k <= j ? 0xff : 0x00;
          expectedData[i * bufferOffset + j * 256 + k * 4 + 2] = 0x00;
          expectedData[i * bufferOffset + j * 256 + k * 4 + 3] = 0xff;
        }
      }
    }

    this.expectGPUBufferValuesEqual(buffer, expectedData);
  }
}

export const g = makeTestGroup(F);

g.test('one_color_attachment,mip_levels').
desc(
  `
  Render to a 3d texture slice with mip levels.
  `
).
params((u) => u.combine('mipLevel', [0, 1, 2]).combine('depthSlice', [0, 1])).
fn((t) => {
  const { mipLevel, depthSlice } = t.params;

  const texture = t.createTextureTracked({
    size: [kSize << mipLevel, kSize << mipLevel, 2 << mipLevel],
    dimension: '3d',
    format: kFormat,
    mipLevelCount: mipLevel + 1,
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  });

  const { bufferSize } = t.getBufferSizeAndOffset(kSize, kSize, 1);

  const buffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const module = t.createShaderModule();

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module },
    fragment: {
      module,
      targets: [{ format: kFormat }]
    },
    primitive: { topology: 'triangle-list' }
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: texture.createView({
        baseMipLevel: mipLevel,
        mipLevelCount: 1
      }),
      depthSlice,
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();
  encoder.copyTextureToBuffer(
    { texture, mipLevel, origin: { x: 0, y: 0, z: depthSlice } },
    { buffer, bytesPerRow: 256 },
    { width: kSize, height: kSize, depthOrArrayLayers: 1 }
  );
  t.device.queue.submit([encoder.finish()]);

  t.checkAttachmentResult(kSize, kSize, 1, buffer);
});

g.test('multiple_color_attachments,same_mip_level').
desc(
  `
  Render to the different slices of 3d texture in multiple color attachments.
  - Same 3d texture with different slices at same mip level
  - Different 3d textures with same slice at same mip level
  `
).
params((u) =>
u.
combine('sameTexture', [true, false]).
beginSubcases().
combine('samePass', [true, false]).
combine('mipLevel', [0, 1])
).
fn((t) => {
  const { sameTexture, samePass, mipLevel } = t.params;

  const formatByteCost = kTextureFormatInfo[kFormat].colorRender.byteCost;
  const maxAttachmentCountPerSample = Math.trunc(
    t.device.limits.maxColorAttachmentBytesPerSample / formatByteCost
  );
  const attachmentCount = Math.min(
    maxAttachmentCountPerSample,
    t.device.limits.maxColorAttachments
  );

  const descriptor = {
    size: [kSize << mipLevel, kSize << mipLevel, 1 << attachmentCount << mipLevel],
    dimension: '3d',
    format: kFormat,
    mipLevelCount: mipLevel + 1,
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  };

  const texture = t.createTextureTracked(descriptor);

  const textures = [];
  const colorAttachments = [];
  for (let i = 0; i < attachmentCount; i++) {
    if (sameTexture) {
      textures.push(texture);
    } else {
      const diffTexture = t.createTextureTracked(descriptor);
      textures.push(diffTexture);
    }

    const colorAttachment = {
      view: textures[i].createView({
        baseMipLevel: mipLevel,
        mipLevelCount: 1
      }),
      depthSlice: sameTexture ? i : 0,
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    };

    colorAttachments.push(colorAttachment);
  }

  const encoder = t.device.createCommandEncoder();

  if (samePass) {
    const module = t.createShaderModule(attachmentCount);

    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module },
      fragment: {
        module,
        targets: new Array(attachmentCount).fill({ format: kFormat })
      },
      primitive: { topology: 'triangle-list' }
    });

    const pass = encoder.beginRenderPass({ colorAttachments });
    pass.setPipeline(pipeline);
    pass.draw(3);
    pass.end();
  } else {
    const module = t.createShaderModule();

    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module },
      fragment: {
        module,
        targets: [{ format: kFormat }]
      },
      primitive: { topology: 'triangle-list' }
    });

    for (let i = 0; i < attachmentCount; i++) {
      const pass = encoder.beginRenderPass({ colorAttachments: [colorAttachments[i]] });
      pass.setPipeline(pipeline);
      pass.draw(3);
      pass.end();
    }
  }

  const { bufferSize, bufferOffset } = t.getBufferSizeAndOffset(kSize, kSize, attachmentCount);
  const buffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  for (let i = 0; i < attachmentCount; i++) {
    encoder.copyTextureToBuffer(
      {
        texture: textures[i],
        mipLevel,
        origin: { x: 0, y: 0, z: sameTexture ? i : 0 }
      },
      { buffer, bytesPerRow: 256, offset: bufferOffset * i },
      { width: kSize, height: kSize, depthOrArrayLayers: 1 }
    );
  }

  t.device.queue.submit([encoder.finish()]);

  t.checkAttachmentResult(kSize, kSize, attachmentCount, buffer);
});

g.test('multiple_color_attachments,same_slice_with_diff_mip_levels').
desc(
  `
  Render to the same slice of a 3d texture at different mip levels in multiple color attachments.
  - For texture size with 1x1xN, the same depth slice of different mip levels can be rendered.
  `
).
params((u) => u.combine('depthSlice', [0, 1])).
fn((t) => {
  const { depthSlice } = t.params;

  const kBaseSize = 1;

  const formatByteCost = kTextureFormatInfo[kFormat].colorRender.byteCost;
  const maxAttachmentCountPerSample = Math.trunc(
    t.device.limits.maxColorAttachmentBytesPerSample / formatByteCost
  );
  const attachmentCount = Math.min(
    maxAttachmentCountPerSample,
    t.device.limits.maxColorAttachments
  );

  const module = t.createShaderModule(attachmentCount);

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module },
    fragment: {
      module,
      targets: new Array(attachmentCount).fill({ format: kFormat })
    },
    primitive: { topology: 'triangle-list' }
  });

  const texture = t.createTextureTracked({
    size: [kBaseSize, kBaseSize, depthSlice + 1 << attachmentCount],
    dimension: '3d',
    format: kFormat,
    mipLevelCount: attachmentCount,
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  });

  const colorAttachments = [];
  for (let i = 0; i < attachmentCount; i++) {
    const colorAttachment = {
      view: texture.createView({
        baseMipLevel: i,
        mipLevelCount: 1
      }),
      depthSlice,
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    };

    colorAttachments.push(colorAttachment);
  }

  const encoder = t.device.createCommandEncoder();

  const pass = encoder.beginRenderPass({ colorAttachments });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();

  const { bufferSize, bufferOffset } = t.getBufferSizeAndOffset(
    kBaseSize,
    kBaseSize,
    attachmentCount
  );
  const buffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  for (let i = 0; i < attachmentCount; i++) {
    encoder.copyTextureToBuffer(
      { texture, mipLevel: i, origin: { x: 0, y: 0, z: depthSlice } },
      { buffer, bytesPerRow: 256, offset: bufferOffset * i },
      { width: kBaseSize, height: kBaseSize, depthOrArrayLayers: 1 }
    );
  }

  t.device.queue.submit([encoder.finish()]);

  t.checkAttachmentResult(kBaseSize, kBaseSize, attachmentCount, buffer);
});