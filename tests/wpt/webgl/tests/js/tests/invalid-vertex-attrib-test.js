var createInvalidAttribTestFn = (function() {

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

function createInvalidAttribTestFn(gl) {
  const vs = `
  attribute vec4 vPosition;
  void main()
  {
      gl_Position = vPosition;
  }
  `;

  const fs = `
  precision mediump float;
  void main()
  {
      gl_FragColor = vec4(1, 0, 0, 1);
  }
  `

  const program = wtu.setupProgram(gl, [vs, fs], ["vPosition"]);
  if (!program) {
    debug(`program failed to compile: ${wtu.getLastError()}`);
  }

  const positionBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
    -1, -1,
     1, -1,
    -1,  1,
    -1,  1,
     1, -1,
     1,  1,
  ]), gl.STATIC_DRAW);

  const indexBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,
                new Uint8Array([0, 1, 2, 3, 4, 5]),
                gl.STATIC_DRAW);

  return async function invalidAttribTestFn(drawFn) {
    debug('');

    // reset attribs
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
    const numAttribs = gl.getParameter(gl.MAX_VERTEX_ATTRIBS);
    for (let i = 0; i < numAttribs; ++i) {
      gl.disableVertexAttribArray(i);
      gl.vertexAttribPointer(1, 1, gl.FLOAT, false, 0, 0);
    }

    debug(`test ${drawFn.name} draws with valid attributes`);
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "there should be no errors");

    gl.clearColor(0, 0, 0, 0,);
    gl.clear(gl.COLOR_BUFFER_BIT);
    wtu.checkCanvas(gl, [0, 0, 0, 0], "canvas should be zero");

    drawFn(gl);

    wtu.checkCanvas(gl, [255, 0, 0, 255], "canvas should be red");
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "there should be no errors");

    debug(`test ${drawFn.name} generates INVALID_OPERATION draws with enabled attribute no buffer bound`);
    gl.enableVertexAttribArray(1);

    gl.clearColor(0, 0, 0, 0,);
    gl.clear(gl.COLOR_BUFFER_BIT);
    wtu.checkCanvas(gl, [0, 0, 0, 0], "canvas should be zero");

    drawFn(gl);

    wtu.glErrorShouldBe(gl, gl.INVALID_OPERATION, "should generate INVALID_OPERATION");
    wtu.checkCanvas(gl, [0, 0, 0, 0], "canvas should be zero");
  };
}

return createInvalidAttribTestFn;
}());
