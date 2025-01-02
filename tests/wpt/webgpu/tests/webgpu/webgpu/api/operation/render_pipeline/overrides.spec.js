/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Testing render pipeline using overridable constants in vertex stage and fragment stage.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';


class F extends GPUTest {
  async ExpectShaderOutputWithConstants(
  isAsync,
  format,
  expected,
  vertex,
  fragment)
  {
    const renderTarget = this.createTextureTracked({
      format,
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
    });

    const descriptor = {
      layout: 'auto',
      vertex,
      fragment,
      primitive: {
        topology: 'triangle-list',
        frontFace: 'ccw',
        cullMode: 'back'
      }
    };

    const promise = isAsync ?
    this.device.createRenderPipelineAsync(descriptor) :
    Promise.resolve(this.device.createRenderPipeline(descriptor));

    const pipeline = await promise;
    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: renderTarget.createView(),
        clearValue: {
          r: kClearValueResult.R,
          g: kClearValueResult.G,
          b: kClearValueResult.B,
          a: kClearValueResult.A
        },
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    pass.setPipeline(pipeline);
    pass.draw(3);
    pass.end();
    this.device.queue.submit([encoder.finish()]);

    this.expectSingleColor(renderTarget, format, {
      size: [1, 1, 1],
      exp: expected
    });
  }
}

export const g = makeTestGroup(F);

const kClearValueResult = { R: 0.2, G: 0.4, B: 0.6, A: 0.8 };
const kDefaultValueResult = { R: 1.0, G: 1.0, B: 1.0, A: 1.0 };

const kFullScreenTriangleVertexShader = `
override xright: f32 = 3.0;
override ytop: f32 = 3.0;

@vertex fn main(
    @builtin(vertex_index) VertexIndex : u32
    ) -> @builtin(position) vec4<f32> {
    var pos : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
        vec2<f32>(-1.0,  ytop),
        vec2<f32>(-1.0,  -ytop),
        vec2<f32>(xright, 0.0));
    return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
}
`;

const kFullScreenTriangleFragmentShader = `
override R: f32 = 1.0;
override G: f32 = 1.0;
override B: f32 = 1.0;
override A: f32 = 1.0;

@fragment fn main()
    -> @location(0) vec4<f32> {
    return vec4<f32>(R, G, B, A);
}
`;

g.test('basic').
desc(
  `Test that either correct constants override values or default values when no constants override value are provided at pipeline creation time are used correctly in vertex and fragment shader.`
).
params((u) =>
u.
combine('isAsync', [true, false]).
beginSubcases().
combineWithParams([
{
  expected: kDefaultValueResult,
  vertexConstants: {},
  fragmentConstants: {}
},
{
  expected: kClearValueResult,
  vertexConstants: {
    xright: -3.0
  },
  fragmentConstants: {}
},
{
  expected: kClearValueResult,
  vertexConstants: {
    ytop: -3.0
  },
  fragmentConstants: {}
},
{
  expected: kDefaultValueResult,
  vertexConstants: {
    xright: 4.0,
    ytop: 4.0
  },
  fragmentConstants: {}
},
{
  expected: { R: 0.0, G: 1.0, B: 0.0, A: 1.0 },
  vertexConstants: {},
  fragmentConstants: { R: 0.0, B: 0.0 }
},
{
  expected: { R: 0.0, G: 0.0, B: 0.0, A: 0.0 },
  vertexConstants: {},
  fragmentConstants: { R: 0.0, G: 0.0, B: 0.0, A: 0.0 }



}]
)
).
fn(async (t) => {
  const format = 'bgra8unorm';
  await t.ExpectShaderOutputWithConstants(
    t.params.isAsync,
    format,
    t.params.expected,
    {
      module: t.device.createShaderModule({
        code: kFullScreenTriangleVertexShader
      }),
      entryPoint: 'main',
      constants: t.params.vertexConstants
    },
    {
      module: t.device.createShaderModule({
        code: kFullScreenTriangleFragmentShader
      }),
      entryPoint: 'main',
      constants: t.params.fragmentConstants,
      targets: [{ format }]
    }
  );
});

g.test('precision').
desc(`Test that the float number precision is preserved for constants`).
params((u) =>
u.
combine('isAsync', [true, false]).
beginSubcases().
combineWithParams([
{
  expected: { R: 3.14159, G: 1.0, B: 1.0, A: 1.0 },
  vertexConstants: {},
  fragmentConstants: { R: 3.14159 }
},
{
  expected: { R: 3.141592653589793, G: 1.0, B: 1.0, A: 1.0 },
  vertexConstants: {},
  fragmentConstants: { R: 3.141592653589793 }
}]
)
).
fn(async (t) => {
  const format = 'rgba32float';
  await t.ExpectShaderOutputWithConstants(
    t.params.isAsync,
    format,
    t.params.expected,
    {
      module: t.device.createShaderModule({
        code: kFullScreenTriangleVertexShader
      }),
      entryPoint: 'main',
      constants: t.params.vertexConstants
    },
    {
      module: t.device.createShaderModule({
        code: kFullScreenTriangleFragmentShader
      }),
      entryPoint: 'main',
      constants: t.params.fragmentConstants,
      targets: [{ format }]
    }
  );
});

g.test('shared_shader_module').
desc(
  `Test that when the same module is shared by different pipelines, the constant values are still being used correctly.`
).
params((u) =>
u.
combine('isAsync', [true, false]).
beginSubcases().
combineWithParams([
{
  expected0: kClearValueResult,
  vertexConstants0: {
    xright: -3.0
  },
  fragmentConstants0: {},

  expected1: kDefaultValueResult,
  vertexConstants1: {},
  fragmentConstants1: {}
},
{
  expected0: { R: 0.0, G: 0.0, B: 0.0, A: 0.0 },
  vertexConstants0: {},
  fragmentConstants0: { R: 0.0, G: 0.0, B: 0.0, A: 0.0 },




  expected1: kDefaultValueResult,
  vertexConstants1: {},
  fragmentConstants1: {}
},
{
  expected0: { R: 1.0, G: 0.0, B: 1.0, A: 0.0 },
  vertexConstants0: {},
  fragmentConstants0: { R: 1.0, G: 0.0, B: 1.0, A: 0.0 },




  expected1: { R: 0.0, G: 1.0, B: 0.0, A: 1.0 },
  vertexConstants1: {},
  fragmentConstants1: { R: 0.0, G: 1.0, B: 0.0, A: 1.0 }



}]
)
).
fn(async (t) => {
  const format = 'bgra8unorm';
  const vertexModule = t.device.createShaderModule({
    code: kFullScreenTriangleVertexShader
  });

  const fragmentModule = t.device.createShaderModule({
    code: kFullScreenTriangleFragmentShader
  });

  const createPipelineFn = async (
  vertexConstants,
  fragmentConstants) =>
  {
    const descriptor = {
      layout: 'auto',
      vertex: {
        module: vertexModule,
        entryPoint: 'main',
        constants: vertexConstants
      },
      fragment: {
        module: fragmentModule,
        entryPoint: 'main',
        targets: [{ format }],
        constants: fragmentConstants
      },
      primitive: {
        topology: 'triangle-list',
        frontFace: 'ccw',
        cullMode: 'back'
      }
    };

    return t.params.isAsync ?
    t.device.createRenderPipelineAsync(descriptor) :
    t.device.createRenderPipeline(descriptor);
  };

  const pipeline0 = await createPipelineFn(
    t.params.vertexConstants0,
    t.params.fragmentConstants0
  );
  const pipeline1 = await createPipelineFn(
    t.params.vertexConstants1,
    t.params.fragmentConstants1
  );

  const renderTarget0 = t.createTextureTracked({
    format,
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });
  const renderTarget1 = t.createTextureTracked({
    format,
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  const encoder = t.device.createCommandEncoder();

  const pass0 = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: renderTarget0.createView(),
      clearValue: {
        r: kClearValueResult.R,
        g: kClearValueResult.G,
        b: kClearValueResult.B,
        a: kClearValueResult.A
      },
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  pass0.setPipeline(pipeline0);
  pass0.draw(3);
  pass0.end();

  const pass1 = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: renderTarget1.createView(),
      clearValue: {
        r: kClearValueResult.R,
        g: kClearValueResult.G,
        b: kClearValueResult.B,
        a: kClearValueResult.A
      },
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  pass1.setPipeline(pipeline1);
  pass1.draw(3);
  pass1.end();

  t.device.queue.submit([encoder.finish()]);

  t.expectSingleColor(renderTarget0, format, {
    size: [1, 1, 1],
    exp: t.params.expected0
  });
  t.expectSingleColor(renderTarget1, format, {
    size: [1, 1, 1],
    exp: t.params.expected1
  });
});

g.test('multi_entry_points').
desc(
  `Test that when the same module is shared by vertex and fragment shader, the constant values are still being used correctly.`
).
params((u) =>
u.
combine('isAsync', [true, false]).
beginSubcases().
combineWithParams([
{
  expected: { R: 0.8, G: 0.4, B: 0.2, A: 1.0 },
  vertexConstants: { A: 4.0, B: 4.0 },
  fragmentConstants: { A: 0.8, B: 0.4, C: 0.2, D: 1.0 }



},
{
  expected: { R: 0.8, G: 0.4, B: 0.2, A: 1.0 },
  vertexConstants: {},
  fragmentConstants: { A: 0.8, B: 0.4, C: 0.2, D: 1.0 }



},
{
  expected: kClearValueResult,
  vertexConstants: { A: -3.0 },
  fragmentConstants: { A: 0.8, B: 0.4, C: 0.2, D: 1.0 }



}]
)
).
fn(async (t) => {
  const format = 'bgra8unorm';
  const module = t.device.createShaderModule({
    code: `
      override A: f32 = 3.0;
      override B: f32 = 3.0;
      override C: f32;
      override D: f32;

      @vertex fn vertexMain(
          @builtin(vertex_index) VertexIndex : u32
          ) -> @builtin(position) vec4<f32> {
          var pos : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
              vec2<f32>(-1.0,  A),
              vec2<f32>(-1.0,  -A),
              vec2<f32>(B, 0.0));
          return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
      }

      @fragment fn fragmentMain()
          -> @location(0) vec4<f32> {
          return vec4<f32>(A, B, C, D);
      }
      `
  });
  await t.ExpectShaderOutputWithConstants(
    t.params.isAsync,
    format,
    t.params.expected,
    {
      module,
      entryPoint: 'vertexMain',
      constants: t.params.vertexConstants
    },
    {
      module,
      entryPoint: 'fragmentMain',
      constants: t.params.fragmentConstants,
      targets: [{ format }]
    }
  );
});