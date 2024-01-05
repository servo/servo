/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import createPatternDataURL from './create-pattern-data-url.js';import { runRefTest } from './gpu_ref_test.js';
runRefTest(async (t) => {
  const { patternSize, imageData: patternImageData } = createPatternDataURL();

  document.querySelector('#dpr').textContent = `dpr: ${devicePixelRatio}`;

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

      @group(0) @binding(0) var pattern: texture_2d<f32>;

      @fragment fn fs(
         @builtin(position) Pos : vec4<f32>
      ) -> @location(0) vec4<f32> {
        let patternSize = textureDimensions(pattern, 0);
        let uPos = vec2u(Pos.xy) % patternSize;
        return textureLoad(pattern, uPos, 0);
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

  const tex = device.createTexture({
    size: [patternSize, patternSize, 1],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST
  });
  device.queue.writeTexture(
    { texture: tex },
    patternImageData.data,
    { bytesPerRow: patternSize * 4, rowsPerImage: 4 },
    { width: patternSize, height: patternSize }
  );

  const bindGroup = device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: tex.createView() }]
  });

  function setCanvasPattern(
  canvas,
  devicePixelWidth,
  devicePixelHeight)
  {
    canvas.width = devicePixelWidth;
    canvas.height = devicePixelHeight;

    const context = canvas.getContext('webgpu');
    context.configure({
      device,
      format: presentationFormat,
      alphaMode: 'premultiplied'
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
    pass.setBindGroup(0, bindGroup);
    pass.draw(3);
    pass.end();

    device.queue.submit([encoder.finish()]);
  }

  /*
  This test creates elements like this
    <body>
      <div class="outer">
        <canvas></canvas>
        <canvas></canvas>
        <canvas></canvas>
        ...
      </div>
    </body>
  Where the outer div is a flexbox centering the child canvases.
  Each of the child canvases is set to a different width in percent.
  The size of each canvas in device pixels is queried with ResizeObserver
  and then each canvases' resolution is set to that size so that there should
  be one pixel in each canvas for each device pixel.
  Each canvas is filled with a pattern using putImageData.
  In the reference the canvas elements are replaced with divs.
  For the divs the same pattern is applied with CSS and its size
  adjusted so the pattern should appear with one pixel in the pattern
  corresponding to 1 device pixel.
  The reference and this page should then match.
  */

  const outerElem = document.querySelector('.outer');

  let resolve;
  const promise = new Promise((_resolve) => resolve = _resolve);

  function setPatternsUsingSizeInfo(entries) {
    for (const entry of entries) {
      setCanvasPattern(
        entry.target,
        entry.devicePixelContentBoxSize[0].inlineSize,
        entry.devicePixelContentBoxSize[0].blockSize
      );
    }
    resolve(true);
  }

  const observer = new ResizeObserver(setPatternsUsingSizeInfo);
  for (let percentSize = 7; percentSize < 100; percentSize += 13) {
    const canvasElem = document.createElement('canvas');
    canvasElem.style.width = `${percentSize}%`;
    observer.observe(canvasElem, { box: 'device-pixel-content-box' });
    outerElem.appendChild(canvasElem);
  }

  await promise;
});