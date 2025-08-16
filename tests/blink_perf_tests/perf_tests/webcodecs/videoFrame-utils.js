function waitForNextFrame() {
  return new Promise((resolve, _) => {
    window.requestAnimationFrame(resolve);
  });
}

function fourColorsFrame(ctx, width, height, text) {
  const kYellow = '#FFFF00';
  const kRed = '#FF0000';
  const kBlue = '#0000FF';
  const kGreen = '#00FF00';

  ctx.fillStyle = kYellow;
  ctx.fillRect(0, 0, width / 2, height / 2);

  ctx.fillStyle = kRed;
  ctx.fillRect(width / 2, 0, width / 2, height / 2);

  ctx.fillStyle = kBlue;
  ctx.fillRect(0, height / 2, width / 2, height / 2);

  ctx.fillStyle = kGreen;
  ctx.fillRect(width / 2, height / 2, width / 2, height / 2);

  ctx.fillStyle = 'white';
  ctx.font = (height / 10) + 'px sans-serif';
  ctx.fillText(text, width / 2, height / 2);
}

async function createDecodedFrame() {
  const config = {codec: 'avc1.64001f', codedWidth: 1280, codedHeight: 720};

  const support = await VideoDecoder.isConfigSupported(config);
  if (!support.supported) {
    PerfTestRunner.logFatalError("Skipping test. Unsupported decoder config:" +
                                 JSON.stringify(config));
    return null;
  }

  const result = await fetch('resources/720p.h264');
  const buf = await result.arrayBuffer();
  const chunk = new EncodedVideoChunk({timestamp: 0, type: 'key', data: buf});

  let frame = null;
  const decoder = new VideoDecoder({
    output: f => frame = f,
    error: e => PerfTestRunner.log('Decode error:' + e)
  });
  decoder.configure(config);
  decoder.decode(chunk);
  await decoder.flush();
  return frame;
}
