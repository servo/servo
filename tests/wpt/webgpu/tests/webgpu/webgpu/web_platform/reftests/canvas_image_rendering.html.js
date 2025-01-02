/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { runRefTest } from './gpu_ref_test.js';runRefTest((t) => {
  const device = t.device;
  const presentationFormat = navigator.gpu.getPreferredCanvasFormat();

  const module = device.createShaderModule({
    code: `
      @vertex fn vs(
        @builtin(vertex_index) VertexIndex : u32
      ) -> @builtin(position) vec4<f32> {
        var pos = array<vec2<f32>, 3>(
        vec2(-1.0, 3.0),
        vec2(-1.0,-1.0),
        vec2( 3.0,-1.0)
        );

        return vec4(pos[VertexIndex], 0.0, 1.0);
      }

      @fragment fn fs(
         @builtin(position) Pos : vec4<f32>
      ) -> @location(0) vec4<f32> {
        let black = vec4f(0, 0, 0, 1);
        let white = vec4f(1, 1, 1, 1);
        let iPos = vec4u(Pos);
        let check = (iPos.x + iPos.y) & 1;
        return mix(black, white, f32(check));
      }
    `
  });

  const pipeline = device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module,
      entryPoint: 'vs'
    },
    fragment: {
      module,
      entryPoint: 'fs',
      targets: [{ format: presentationFormat }]
    }
  });

  function draw(selector, alphaMode) {
    const canvas = document.querySelector(selector);
    const context = canvas.getContext('webgpu');
    context.configure({
      device,
      format: presentationFormat,
      alphaMode
    });

    const encoder = device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: context.getCurrentTexture().createView(),
        clearValue: [0.0, 0.0, 0.0, 0.0],
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    pass.setPipeline(pipeline);
    pass.draw(3);
    pass.end();

    device.queue.submit([encoder.finish()]);
  }

  draw('#elem1', 'premultiplied');
  draw('#elem2', 'premultiplied');
  draw('#elem3', 'premultiplied');
  draw('#elem4', 'opaque');
  draw('#elem5', 'opaque');
  draw('#elem6', 'opaque');
});