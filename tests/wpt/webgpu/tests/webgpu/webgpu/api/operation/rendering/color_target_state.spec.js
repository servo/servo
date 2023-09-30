/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Test blending results.

TODO:
- Test result for all combinations of args (make sure each case is distinguishable from others
- Test underflow/overflow has consistent behavior
- ?
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert, unreachable } from '../../../../common/util/util.js';
import { kBlendFactors, kBlendOperations } from '../../../capability_info.js';
import { GPUConst } from '../../../constants.js';
import { kEncodableTextureFormats, kTextureFormatInfo } from '../../../format_info.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';
import { clamp } from '../../../util/math.js';
import { TexelView } from '../../../util/texture/texel_view.js';

class BlendingTest extends GPUTest {
  createRenderPipelineForTest(colorTargetState) {
    return this.device.createRenderPipeline({
      layout: 'auto',
      fragment: {
        targets: [colorTargetState],
        module: this.device.createShaderModule({
          code: `
            struct Params {
              color : vec4<f32>
            }
            @group(0) @binding(0) var<uniform> params : Params;
            @fragment fn main() -> @location(0) vec4<f32> {
              return params.color;
            }
            `,
        }),
        entryPoint: 'main',
      },
      vertex: {
        module: this.device.createShaderModule({
          code: `
            @vertex fn main(
              @builtin(vertex_index) VertexIndex : u32
              ) -> @builtin(position) vec4<f32> {
              var pos = array<vec2<f32>, 3>(
                  vec2<f32>(-1.0, -1.0),
                  vec2<f32>(3.0, -1.0),
                  vec2<f32>(-1.0, 3.0));
              return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
            }
            `,
        }),
        entryPoint: 'main',
      },
    });
  }

  createBindGroupForTest(layout, data) {
    return this.device.createBindGroup({
      layout,
      entries: [
        {
          binding: 0,
          resource: {
            buffer: this.makeBufferWithContents(data, GPUBufferUsage.UNIFORM),
          },
        },
      ],
    });
  }
}

export const g = makeTestGroup(TextureTestMixin(BlendingTest));

function mapColor(col, f) {
  return {
    r: f(col.r, 'r'),
    g: f(col.g, 'g'),
    b: f(col.b, 'b'),
    a: f(col.a, 'a'),
  };
}

function computeBlendFactor(src, dst, blendColor, factor) {
  switch (factor) {
    case 'zero':
      return { r: 0, g: 0, b: 0, a: 0 };
    case 'one':
      return { r: 1, g: 1, b: 1, a: 1 };
    case 'src':
      return { ...src };
    case 'one-minus-src':
      return mapColor(src, v => 1 - v);
    case 'src-alpha':
      return mapColor(src, () => src.a);
    case 'one-minus-src-alpha':
      return mapColor(src, () => 1 - src.a);
    case 'dst':
      return { ...dst };
    case 'one-minus-dst':
      return mapColor(dst, v => 1 - v);
    case 'dst-alpha':
      return mapColor(dst, () => dst.a);
    case 'one-minus-dst-alpha':
      return mapColor(dst, () => 1 - dst.a);
    case 'src-alpha-saturated': {
      const f = Math.min(src.a, 1 - dst.a);
      return { r: f, g: f, b: f, a: 1 };
    }
    case 'constant':
      assert(blendColor !== undefined);
      return { ...blendColor };
    case 'one-minus-constant':
      assert(blendColor !== undefined);
      return mapColor(blendColor, v => 1 - v);
    default:
      unreachable();
  }
}

function computeBlendOperation(src, srcFactor, dst, dstFactor, operation) {
  switch (operation) {
    case 'add':
      return mapColor(src, (_, k) => srcFactor[k] * src[k] + dstFactor[k] * dst[k]);
    case 'max':
      return mapColor(src, (_, k) => Math.max(src[k], dst[k]));
    case 'min':
      return mapColor(src, (_, k) => Math.min(src[k], dst[k]));
    case 'reverse-subtract':
      return mapColor(src, (_, k) => dstFactor[k] * dst[k] - srcFactor[k] * src[k]);
    case 'subtract':
      return mapColor(src, (_, k) => srcFactor[k] * src[k] - dstFactor[k] * dst[k]);
  }
}

g.test('blending,GPUBlendComponent')
  .desc(
    `Test all combinations of parameters for GPUBlendComponent.

  Tests that parameters are correctly passed to the backend API and blend computations
  are done correctly by blending a single pixel. The test uses rgba16float as the format
  to avoid checking clamping behavior (tested in api,operation,rendering,blending:clamp,*).

  Params:
    - component= {color, alpha} - whether to test blending the color or the alpha component.
    - srcFactor= {...all GPUBlendFactors}
    - dstFactor= {...all GPUBlendFactors}
    - operation= {...all GPUBlendOperations}`
  )
  .params(u =>
    u //
      .combine('component', ['color', 'alpha'])
      .combine('srcFactor', kBlendFactors)
      .combine('dstFactor', kBlendFactors)
      .combine('operation', kBlendOperations)
      .filter(t => {
        if (t.operation === 'min' || t.operation === 'max') {
          return t.srcFactor === 'one' && t.dstFactor === 'one';
        }
        return true;
      })
      .beginSubcases()
      .combine('srcColor', [{ r: 0.11, g: 0.61, b: 0.81, a: 0.44 }])
      .combine('dstColor', [
        { r: 0.51, g: 0.22, b: 0.71, a: 0.33 },
        { r: 0.09, g: 0.73, b: 0.93, a: 0.81 },
      ])
      .expand('blendConstant', p => {
        const needsBlendConstant =
          p.srcFactor === 'one-minus-constant' ||
          p.srcFactor === 'constant' ||
          p.dstFactor === 'one-minus-constant' ||
          p.dstFactor === 'constant';
        return needsBlendConstant ? [{ r: 0.91, g: 0.82, b: 0.73, a: 0.64 }] : [undefined];
      })
  )
  .fn(t => {
    const textureFormat = 'rgba16float';
    const srcColor = t.params.srcColor;
    const dstColor = t.params.dstColor;
    const blendConstant = t.params.blendConstant;

    const srcFactor = computeBlendFactor(srcColor, dstColor, blendConstant, t.params.srcFactor);
    const dstFactor = computeBlendFactor(srcColor, dstColor, blendConstant, t.params.dstFactor);

    const expectedColor = computeBlendOperation(
      srcColor,
      srcFactor,
      dstColor,
      dstFactor,
      t.params.operation
    );

    switch (t.params.component) {
      case 'color':
        expectedColor.a = srcColor.a;
        break;
      case 'alpha':
        expectedColor.r = srcColor.r;
        expectedColor.g = srcColor.g;
        expectedColor.b = srcColor.b;
        break;
    }

    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      fragment: {
        targets: [
          {
            format: textureFormat,
            blend: {
              // Set both color/alpha to defaults...
              color: {},
              alpha: {},
              // ... but then override the component we're testing.
              [t.params.component]: {
                srcFactor: t.params.srcFactor,
                dstFactor: t.params.dstFactor,
                operation: t.params.operation,
              },
            },
          },
        ],

        module: t.device.createShaderModule({
          code: `
struct Uniform {
  color: vec4<f32>
};
@group(0) @binding(0) var<uniform> u : Uniform;

@fragment fn main() -> @location(0) vec4<f32> {
  return u.color;
}
          `,
        }),
        entryPoint: 'main',
      },
      vertex: {
        module: t.device.createShaderModule({
          code: `
@vertex fn main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
          `,
        }),
        entryPoint: 'main',
      },
      primitive: {
        topology: 'point-list',
      },
    });

    const renderTarget = t.device.createTexture({
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      size: [1, 1, 1],
      format: textureFormat,
    });

    const commandEncoder = t.device.createCommandEncoder();
    const renderPass = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: renderTarget.createView(),
          clearValue: dstColor,
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
    });
    renderPass.setPipeline(pipeline);
    if (blendConstant) {
      renderPass.setBlendConstant(blendConstant);
    }
    renderPass.setBindGroup(
      0,
      t.device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
          {
            binding: 0,
            resource: {
              buffer: t.makeBufferWithContents(
                new Float32Array([srcColor.r, srcColor.g, srcColor.b, srcColor.a]),
                GPUBufferUsage.UNIFORM
              ),
            },
          },
        ],
      })
    );

    renderPass.draw(1);
    renderPass.end();

    t.device.queue.submit([commandEncoder.finish()]);

    t.expectSinglePixelComparisonsAreOkInTexture(
      { texture: renderTarget },
      [
        {
          coord: { x: 0, y: 0 },
          exp: { R: expectedColor.r, G: expectedColor.g, B: expectedColor.b, A: expectedColor.a },
        },
      ],

      { maxFractionalDiff: 0.003 }
    );
  });

const kBlendableFormats = kEncodableTextureFormats.filter(f => {
  const info = kTextureFormatInfo[f];
  return info.renderable && info.sampleType === 'float';
});

g.test('blending,formats')
  .desc(
    `Test blending results works for all formats that support it, and that blending is not applied
  for formats that do not. Blending should be done in linear space for srgb formats.`
  )
  .params(u =>
    u //
      .combine('format', kBlendableFormats)
  )
  .beforeAllSubcases(t => {
    t.skipIfTextureFormatNotSupported(t.params.format);
  })
  .fn(t => {
    const { format } = t.params;

    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      fragment: {
        targets: [
          {
            format,
            blend: {
              color: { srcFactor: 'one', dstFactor: 'one', operation: 'add' },
              alpha: { srcFactor: 'one', dstFactor: 'one', operation: 'add' },
            },
          },
        ],

        module: t.device.createShaderModule({
          code: `
@fragment fn main() -> @location(0) vec4<f32> {
  return vec4<f32>(0.4, 0.4, 0.4, 0.4);
}
          `,
        }),
        entryPoint: 'main',
      },
      vertex: {
        module: t.device.createShaderModule({
          code: `
@vertex fn main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
          `,
        }),
        entryPoint: 'main',
      },
      primitive: {
        topology: 'point-list',
      },
    });

    const renderTarget = t.device.createTexture({
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      size: [1, 1, 1],
      format,
    });

    const commandEncoder = t.device.createCommandEncoder();
    const renderPass = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: renderTarget.createView(),
          clearValue: { r: 0.2, g: 0.2, b: 0.2, a: 0.2 },
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
    });
    renderPass.setPipeline(pipeline);
    renderPass.draw(1);
    renderPass.end();
    t.device.queue.submit([commandEncoder.finish()]);

    const expColor = { R: 0.6, G: 0.6, B: 0.6, A: 0.6 };
    const expTexelView = TexelView.fromTexelsAsColors(format, coords => expColor);
    t.expectTexelViewComparisonIsOkInTexture({ texture: renderTarget }, expTexelView, [1, 1, 1]);
  });

g.test('blend_constant,initial')
  .desc(`Test that the blend constant is set to [0,0,0,0] at the beginning of a pass.`)
  .fn(t => {
    const format = 'rgba8unorm';
    const kSize = 1;
    const kWhiteColorData = new Float32Array([255, 255, 255, 255]);

    const blendComponent = { srcFactor: 'constant', dstFactor: 'one', operation: 'add' };
    const testPipeline = t.createRenderPipelineForTest({
      format,
      blend: { color: blendComponent, alpha: blendComponent },
    });

    const renderTarget = t.device.createTexture({
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      size: [kSize, kSize],
      format,
    });

    const commandEncoder = t.device.createCommandEncoder();
    const renderPass = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: renderTarget.createView(),
          loadOp: 'load',
          storeOp: 'store',
        },
      ],
    });
    renderPass.setPipeline(testPipeline);
    renderPass.setBindGroup(
      0,
      t.createBindGroupForTest(testPipeline.getBindGroupLayout(0), kWhiteColorData)
    );

    renderPass.draw(3);
    // Draw [1,1,1,1] with `src * constant + dst * 1`.
    // The blend constant defaults to [0,0,0,0], so the result is
    // `[1,1,1,1] * [0,0,0,0] + [0,0,0,0] * 1` = [0,0,0,0].
    renderPass.end();
    t.device.queue.submit([commandEncoder.finish()]);

    // Check that the initial blend constant is black(0,0,0,0) after setting testPipeline which has
    // a white color buffer data.
    const expColor = { R: 0, G: 0, B: 0, A: 0 };
    const expTexelView = TexelView.fromTexelsAsColors(format, coords => expColor);
    t.expectTexelViewComparisonIsOkInTexture({ texture: renderTarget }, expTexelView, [
      kSize,
      kSize,
    ]);
  });

g.test('blend_constant,setting')
  .desc(`Test that setting the blend constant to the RGBA values works at the beginning of a pass.`)
  .paramsSubcasesOnly([
    { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
    { r: 0.5, g: 1.0, b: 0.5, a: 0.0 },
    { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
  ])
  .fn(t => {
    const { r, g, b, a } = t.params;

    const format = 'rgba8unorm';
    const kSize = 1;
    const kWhiteColorData = new Float32Array([255, 255, 255, 255]);

    const blendComponent = { srcFactor: 'constant', dstFactor: 'one', operation: 'add' };
    const testPipeline = t.createRenderPipelineForTest({
      format,
      blend: { color: blendComponent, alpha: blendComponent },
    });

    const renderTarget = t.device.createTexture({
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      size: [kSize, kSize],
      format,
    });

    const commandEncoder = t.device.createCommandEncoder();
    const renderPass = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: renderTarget.createView(),
          loadOp: 'load',
          storeOp: 'store',
        },
      ],
    });
    renderPass.setPipeline(testPipeline);
    renderPass.setBlendConstant({ r, g, b, a });
    renderPass.setBindGroup(
      0,
      t.createBindGroupForTest(testPipeline.getBindGroupLayout(0), kWhiteColorData)
    );

    renderPass.draw(3);
    // Draw [1,1,1,1] with `src * constant + dst * 1`. The blend constant to [r,g,b,a], so the
    // result is `[1,1,1,1] * [r,g,b,a] + [0,0,0,0] * 1` = [r,g,b,a].
    renderPass.end();
    t.device.queue.submit([commandEncoder.finish()]);

    // Check that the blend constant is the same as the given constant after setting the constant
    // via setBlendConstant.
    const expColor = { R: r, G: g, B: b, A: a };
    const expTexelView = TexelView.fromTexelsAsColors(format, coords => expColor);

    t.expectTexelViewComparisonIsOkInTexture({ texture: renderTarget }, expTexelView, [
      kSize,
      kSize,
    ]);
  });

g.test('blend_constant,not_inherited')
  .desc(`Test that the blending constant is not inherited between render passes.`)
  .fn(t => {
    const format = 'rgba8unorm';
    const kSize = 1;
    const kWhiteColorData = new Float32Array([255, 255, 255, 255]);

    const blendComponent = { srcFactor: 'constant', dstFactor: 'one', operation: 'add' };
    const testPipeline = t.createRenderPipelineForTest({
      format,
      blend: { color: blendComponent, alpha: blendComponent },
    });

    const renderTarget = t.device.createTexture({
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      size: [kSize, kSize],
      format,
    });

    const commandEncoder = t.device.createCommandEncoder();
    {
      const renderPass = commandEncoder.beginRenderPass({
        colorAttachments: [
          {
            view: renderTarget.createView(),
            loadOp: 'load',
            storeOp: 'store',
          },
        ],
      });
      renderPass.setPipeline(testPipeline);
      renderPass.setBlendConstant({ r: 1.0, g: 1.0, b: 1.0, a: 1.0 }); // Set to white color.
      renderPass.setBindGroup(
        0,
        t.createBindGroupForTest(testPipeline.getBindGroupLayout(0), kWhiteColorData)
      );

      renderPass.draw(3);
      // Draw [1,1,1,1] with `src * constant + dst * 1`. The blend constant to [1,1,1,1], so the
      // result is `[1,1,1,1] * [1,1,1,1] + [0,0,0,0] * 1` = [1,1,1,1].
      renderPass.end();
    }
    {
      const renderPass = commandEncoder.beginRenderPass({
        colorAttachments: [
          {
            view: renderTarget.createView(),
            loadOp: 'clear',
            storeOp: 'store',
          },
        ],
      });
      renderPass.setPipeline(testPipeline);
      renderPass.setBindGroup(
        0,
        t.createBindGroupForTest(testPipeline.getBindGroupLayout(0), kWhiteColorData)
      );

      renderPass.draw(3);
      // Draw [1,1,1,1] with `src * constant + dst * 1`. The blend constant defaults to [0,0,0,0],
      // so the result is `[1,1,1,1] * [0,0,0,0] + [0,0,0,0] * 1` = [0,0,0,0].
      renderPass.end();
    }
    t.device.queue.submit([commandEncoder.finish()]);

    // Check that the blend constant is not inherited from the first render pass.
    const expColor = { R: 0, G: 0, B: 0, A: 0 };
    const expTexelView = TexelView.fromTexelsAsColors(format, coords => expColor);

    t.expectTexelViewComparisonIsOkInTexture({ texture: renderTarget }, expTexelView, [
      kSize,
      kSize,
    ]);
  });

const kColorWriteCombinations = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

g.test('color_write_mask,channel_work')
  .desc(
    `
  Test that the color write mask works with the zero channel, a single channel, multiple channels,
  and all channels.
  `
  )
  .params(u =>
    u //
      .combine('mask', kColorWriteCombinations)
  )
  .fn(t => {
    const { mask } = t.params;

    const format = 'rgba8unorm';
    const kSize = 1;

    let r = 0,
      g = 0,
      b = 0,
      a = 0;
    if (mask & GPUConst.ColorWrite.RED) {
      r = 1;
    }
    if (mask & GPUConst.ColorWrite.GREEN) {
      g = 1;
    }
    if (mask & GPUConst.ColorWrite.BLUE) {
      b = 1;
    }
    if (mask & GPUConst.ColorWrite.ALPHA) {
      a = 1;
    }

    const testPipeline = t.createRenderPipelineForTest({
      format,
      writeMask: mask,
    });

    const renderTarget = t.device.createTexture({
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      size: [kSize, kSize],
      format,
    });

    const kBaseColorData = new Float32Array([32, 64, 128, 192]);

    const commandEncoder = t.device.createCommandEncoder();
    {
      const renderPass = commandEncoder.beginRenderPass({
        colorAttachments: [
          {
            view: renderTarget.createView(),
            loadOp: 'load',
            storeOp: 'store',
          },
        ],
      });
      renderPass.setPipeline(testPipeline);
      renderPass.setBindGroup(
        0,
        t.createBindGroupForTest(testPipeline.getBindGroupLayout(0), kBaseColorData)
      );

      renderPass.draw(3);
      renderPass.end();
    }
    t.device.queue.submit([commandEncoder.finish()]);

    const expColor = { R: r, G: g, B: b, A: a };
    const expTexelView = TexelView.fromTexelsAsColors(format, coords => expColor);

    t.expectTexelViewComparisonIsOkInTexture({ texture: renderTarget }, expTexelView, [
      kSize,
      kSize,
    ]);
  });

g.test('color_write_mask,blending_disabled')
  .desc(
    `Test that the color write mask works when blending is disabled or set to the defaults
  (which has the same blending result).`
  )
  .params(u => u.combine('disabled', [false, true]))
  .fn(t => {
    const format = 'rgba8unorm';
    const kSize = 1;

    const blend = t.params.disabled ? undefined : { color: {}, alpha: {} };

    const testPipeline = t.createRenderPipelineForTest({
      format,
      blend,
      writeMask: GPUColorWrite.RED,
    });

    const renderTarget = t.device.createTexture({
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      size: [kSize, kSize],
      format,
    });

    const kBaseColorData = new Float32Array([32, 64, 128, 192]);

    const commandEncoder = t.device.createCommandEncoder();
    {
      const renderPass = commandEncoder.beginRenderPass({
        colorAttachments: [
          {
            view: renderTarget.createView(),
            loadOp: 'load',
            storeOp: 'store',
          },
        ],
      });
      renderPass.setPipeline(testPipeline);
      renderPass.setBindGroup(
        0,
        t.createBindGroupForTest(testPipeline.getBindGroupLayout(0), kBaseColorData)
      );

      // Draw [1,1,1,1] with `src * 1 + dst * 0`. So the
      // result is `[1,1,1,1] * [1,1,1,1] + [0,0,0,0] * 0` = [1,1,1,1].
      renderPass.draw(3);
      renderPass.end();
    }
    t.device.queue.submit([commandEncoder.finish()]);

    const expColor = { R: 1, G: 0, B: 0, A: 0 };
    const expTexelView = TexelView.fromTexelsAsColors(format, coords => expColor);

    t.expectTexelViewComparisonIsOkInTexture({ texture: renderTarget }, expTexelView, [
      kSize,
      kSize,
    ]);
  });

g.test('blending,clamping')
  .desc(
    `
  Test that clamping occurs at the correct points in the blend process: src value, src factor, dst
  factor, and output.
    - TODO: Need to test snorm formats.
    - TODO: Need to test src value, srcFactor and dstFactor.
  `
  )
  .params(u =>
    u //
      .combine('format', ['rgba8unorm', 'rg16float'])
      .combine('srcValue', [0.4, 0.6, 0.8, 1.0])
      .combine('dstValue', [0.2, 0.4])
  )
  .fn(t => {
    const { format, srcValue, dstValue } = t.params;

    const blendComponent = { srcFactor: 'one', dstFactor: 'one', operation: 'add' };

    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      fragment: {
        targets: [
          {
            format,
            blend: {
              color: blendComponent,
              alpha: blendComponent,
            },
          },
        ],

        module: t.device.createShaderModule({
          code: `
@fragment fn main() -> @location(0) vec4<f32> {
  return vec4<f32>(${srcValue}, ${srcValue}, ${srcValue}, ${srcValue});
}
          `,
        }),
        entryPoint: 'main',
      },
      vertex: {
        module: t.device.createShaderModule({
          code: `
@vertex fn main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
          `,
        }),
        entryPoint: 'main',
      },
      primitive: {
        topology: 'point-list',
      },
    });

    const renderTarget = t.device.createTexture({
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      size: [1, 1, 1],
      format,
    });

    const commandEncoder = t.device.createCommandEncoder();
    const renderPass = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: renderTarget.createView(),
          clearValue: { r: dstValue, g: dstValue, b: dstValue, a: dstValue },
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
    });
    renderPass.setPipeline(pipeline);
    renderPass.draw(1);
    renderPass.end();
    t.device.queue.submit([commandEncoder.finish()]);

    let expValue;
    switch (format) {
      case 'rgba8unorm': // unorm types should clamp if the sum of srcValue and dstValue exceeds 1.
        expValue = clamp(srcValue + dstValue, { min: 0, max: 1 });
        break;
      case 'rg16float': // float format types doesn't clamp.
        expValue = srcValue + dstValue;
        break;
    }

    const expColor = { R: expValue, G: expValue, B: expValue, A: expValue };
    const expTexelView = TexelView.fromTexelsAsColors(format, coords => expColor);

    t.expectTexelViewComparisonIsOkInTexture({ texture: renderTarget }, expTexelView, [1, 1, 1]);
  });
