const TILE_SIZE = 3;
const SCALE = 20;
function createTileCanvas() {
  const canvas = document.createElement('canvas');
  canvas.width = TILE_SIZE;
  canvas.height = TILE_SIZE;
  return canvas;
}

function fillUnsupported(canvas) {
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = 'rgb(255, 0, 255)';
  ctx.fillRect(0, 0, TILE_SIZE, TILE_SIZE);
}

function fillError() {
  const output = document.getElementById('output');
  output.width = 40;
  output.height = 40;
  const ctx = output.getContext('2d');
  ctx.fillStyle = 'rgb(255, 0, 0)';
  ctx.fillRect(0, 0, output.width, output.height);
}

async function loadImage(src) {
  const image = new Image();
  await new Promise((resolve, reject) => {
    image.onload = resolve;
    image.onerror = () => reject(new Error(`image load failed: ${src}`));
    image.src = src;
  });
  return image;
}

async function renderDrawImageTile(image) {
  const canvas = createTileCanvas();
  const ctx = canvas.getContext('2d');
  ctx.drawImage(image, 0, 0);
  return canvas;
}

async function renderImageBitmapTile(src) {
  const canvas = createTileCanvas();
  const ctx = canvas.getContext('2d');

  try {
    const response = await fetch(src);
    const blob = await response.blob();
    const bitmap = await createImageBitmap(blob);
    ctx.drawImage(bitmap, 0, 0);
    bitmap.close();
    return canvas;
  } catch (error) {
    fillUnsupported(canvas);
    return canvas;
  }
}

function makeImageDataFromWebGLPixels(pixels) {
  const flipped = new Uint8ClampedArray(pixels.length);
  const rowWidth = TILE_SIZE * 4;
  for (let y = 0; y < TILE_SIZE; ++y) {
    const sourceOffset = (TILE_SIZE - 1 - y) * rowWidth;
    const destinationOffset = y * rowWidth;
    flipped.set(pixels.subarray(sourceOffset, sourceOffset + rowWidth),
                destinationOffset);
  }
  return new ImageData(flipped, TILE_SIZE, TILE_SIZE);
}

function renderWebGLTile(image) {
  const outputCanvas = createTileCanvas();
  const outputContext = outputCanvas.getContext('2d');

  const webglCanvas = createTileCanvas();
  const gl = webglCanvas.getContext('webgl');
  if (!gl) {
    fillUnsupported(outputCanvas);
    return outputCanvas;
  }

  const texture = gl.createTexture();
  const framebuffer = gl.createFramebuffer();
  if (!texture || !framebuffer) {
    fillUnsupported(outputCanvas);
    return outputCanvas;
  }

  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

  try {
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
  } catch (error) {
    fillUnsupported(outputCanvas);
    gl.deleteFramebuffer(framebuffer);
    gl.deleteTexture(texture);
    return outputCanvas;
  }

  gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
  gl.framebufferTexture2D(
      gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);

  if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE) {
    fillUnsupported(outputCanvas);
    gl.deleteFramebuffer(framebuffer);
    gl.deleteTexture(texture);
    return outputCanvas;
  }

  const pixels = new Uint8Array(TILE_SIZE * TILE_SIZE * 4);
  gl.readPixels(0, 0, TILE_SIZE, TILE_SIZE, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

  const imageData = makeImageDataFromWebGLPixels(pixels);
  outputContext.putImageData(imageData, 0, 0);

  gl.deleteFramebuffer(framebuffer);
  gl.deleteTexture(texture);
  return outputCanvas;
}

function convertFloatChannelToByte(value) {
  if (!Number.isFinite(value)) {
    return 0;
  }
  if (value <= 0) {
    return 0;
  }
  if (value >= 1) {
    return 255;
  }
  return Math.round(value * 255);
}

function renderFloat16Tile(image) {
  const canvas = createTileCanvas();
  const ctx = canvas.getContext('2d');
  ctx.drawImage(image, 0, 0);

  let floatData;
  try {
    floatData =
        ctx.getImageData(0, 0, TILE_SIZE, TILE_SIZE,
                         {pixelFormat: 'rgba-float16'}).data;
  } catch (error) {
    fillUnsupported(canvas);
    return canvas;
  }

  if (!(floatData instanceof Float16Array)) {
    fillUnsupported(canvas);
    return canvas;
  }

  const converted = new Uint8ClampedArray(floatData.length);
  for (let i = 0; i < floatData.length; ++i) {
    converted[i] = convertFloatChannelToByte(floatData[i]);
  }

  ctx.putImageData(new ImageData(converted, TILE_SIZE, TILE_SIZE), 0, 0);
  return canvas;
}

function drawOutputTiles(tiles) {
  const output = document.getElementById('output');
  output.width = TILE_SIZE * SCALE * tiles.length;
  output.height = TILE_SIZE * SCALE;

  const ctx = output.getContext('2d');
  ctx.imageSmoothingEnabled = false;

  const scaledTileSize = TILE_SIZE * SCALE;
  for (let i = 0; i < tiles.length; ++i) {
    ctx.drawImage(
        tiles[i],
        0,
        0,
        TILE_SIZE,
        TILE_SIZE,
        i * scaledTileSize,
        0,
        scaledTileSize,
        scaledTileSize);
  }
}

async function runCanvasDecodePathReftest(source) {
  try {
    const image = await loadImage(source);
    const drawImageTile = await renderDrawImageTile(image);
    const imageBitmapTile = await renderImageBitmapTile(source);
    const webglTile = renderWebGLTile(image);
    const float16Tile = renderFloat16Tile(image);
    drawOutputTiles([drawImageTile, imageBitmapTile, webglTile, float16Tile]);
  } catch (error) {
    fillError();
  } finally {
    document.documentElement.classList.remove('reftest-wait');
  }
}
