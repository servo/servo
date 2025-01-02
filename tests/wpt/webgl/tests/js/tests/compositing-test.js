var createCompositingTestFn = (function() {

const width = 20;
const height = 20;

function waitForComposite() {
  debug('wait for composite');
  return new Promise(resolve => wtu.waitForComposite(resolve));
}

async function testPreserveDrawingBufferFalse(gl, drawFn, clear) {
  debug('');
  debug(`test preserveDrawingBuffer: false with ${drawFn.name} ${clear ? 'with' : 'without'} clear`);

  if (clear) {
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
  }

  if (drawFn(gl)) {
    debug('skipped: extension does not exist');
    return;
  }

  wtu.checkCanvas(gl, [255, 0, 0, 255], "canvas should be red");

  // enable scissor here, before compositing, to make sure it's correctly
  // ignored and restored
  const halfWidth = gl.canvas.width / 2;
  const halfHeight = gl.canvas.height / 2;
  gl.scissor(0, halfHeight, halfWidth, halfHeight);
  gl.enable(gl.SCISSOR_TEST);

  await waitForComposite();

  // scissor was set earlier
  gl.clearColor(0, 0, 1, 1);
  gl.clear(gl.COLOR_BUFFER_BIT);

  wtu.checkCanvasRect(gl, 0, halfHeight, halfWidth, halfHeight, [0, 0, 255, 255],
      "cleared corner should be blue, stencil should be preserved");
  wtu.checkCanvasRect(gl, 0, 0, halfWidth, halfHeight, [0, 0, 0, 0],
      "remainder of buffer should be cleared");

  gl.disable(gl.SCISSOR_TEST);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR, "there should be no errors");
}

async function testPreserveDrawingBufferTrue(gl, drawFn, clear) {
  debug('');
  debug(`test preserveDrawingBuffer: true with ${drawFn.name} ${clear ? 'with' : 'without'} clear`);

  if (clear) {
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
  }

  const skipTest = drawFn(gl);
  if (skipTest) {
    debug('skipped: extension does not exist');
    return;
  }

  wtu.checkCanvas(gl, [255, 0, 0, 255], "canvas should be red");

  await waitForComposite();

  wtu.checkCanvas(gl, [255, 0, 0, 255], "canvas should be red");
  wtu.glErrorShouldBe(gl, gl.NO_ERROR, "there should be no errors");
}

function setupWebGL({
    webglVersion,
    shadersFn,
    attribs,
}) {
    const existingCanvases = document.querySelectorAll('canvas');
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    canvas.style.display = 'block';
    canvas.style.position = 'fixed';
    canvas.style.left = `${existingCanvases.length * 25}px`;
    canvas.style.top = '0';
    // The canvas needs to be visible or the test will fail.
    document.body.insertBefore(canvas, [...existingCanvases].pop());
    const gl = wtu.create3DContext(canvas, attribs, webglVersion);
    if (!gl) {
      testFailed('WebGL context creation failed');
      return gl;
    }

    const shaders = shadersFn(gl);
    const program = wtu.setupProgram(gl, shaders, ["position"]);
    if (!program) {
      debug(`program failed to compile: ${wtu.getLastError()}`);
    }
    const positionBuf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
      -1, -1,
       1, -1,
      -1,  1,
      -1,  1,
       1, -1,
       1,  1,
    ]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    const indexBuf = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuf);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint8Array([0, 1, 2, 3, 4, 5]), gl.STATIC_DRAW);
    return gl;
}

function createCompositingTestFn(options) {
  const glPreserveDrawingBufferFalse = setupWebGL({
    ...options,
    attribs: {antialias: false},
  });
  const glPreserveDrawingBufferTrue = setupWebGL({
    ...options,
    attribs: {antialias: false, preserveDrawingBuffer: true},
  });
  return async function(drawFn) {
    debug('---');
    await testPreserveDrawingBufferFalse(glPreserveDrawingBufferFalse, drawFn, false);
    await testPreserveDrawingBufferFalse(glPreserveDrawingBufferFalse, drawFn, true);

    await testPreserveDrawingBufferTrue(glPreserveDrawingBufferTrue, drawFn, false);
    await testPreserveDrawingBufferTrue(glPreserveDrawingBufferTrue, drawFn, true);
  };
}

return createCompositingTestFn;
}());
