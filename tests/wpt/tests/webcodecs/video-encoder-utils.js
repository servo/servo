async function checkEncoderSupport(test, config) {
  assert_equals("function", typeof VideoEncoder.isConfigSupported);
  let supported = false;
  try {
    const support = await VideoEncoder.isConfigSupported(config);
    supported = support.supported;
  } catch (e) {}

  assert_implements_optional(supported, 'Unsupported config: ' +
                             JSON.stringify(config));
}

function fourColorsFrame(ctx, width, height, text) {
  const kYellow = "#FFFF00";
  const kRed = "#FF0000";
  const kBlue = "#0000FF";
  const kGreen = "#00FF00";

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

// Paints |count| black dots on the |ctx|, so their presence can be validated
// later. This is an analog of the most basic bar code.
function putBlackDots(ctx, width, height, count) {
  ctx.fillStyle = 'black';
  const dot_size = 20;
  const step = dot_size * 2;

  for (let i = 1; i <= count; i++) {
    let x = i * step;
    let y = step * (x / width + 1);
    x %= width;
    ctx.fillRect(x, y, dot_size, dot_size);
  }
}

// Validates that frame has |count| black dots in predefined places.
function validateBlackDots(frame, count) {
  const width = frame.displayWidth;
  const height = frame.displayHeight;
  let cnv = new OffscreenCanvas(width, height);
  var ctx = cnv.getContext('2d', {willReadFrequently: true});
  ctx.drawImage(frame, 0, 0);
  const dot_size = 20;
  const step = dot_size * 2;

  for (let i = 1; i <= count; i++) {
    let x = i * step + dot_size / 2;
    let y = step * (x / width + 1) + dot_size / 2;
    x %= width;

    if (x)
      x = x -1;
    if (y)
      y = y -1;

    let rgba = ctx.getImageData(x, y, 2, 2).data;
    const tolerance = 60;
    if ((rgba[0] > tolerance || rgba[1] > tolerance || rgba[2] > tolerance)
      && (rgba[4] > tolerance || rgba[5] > tolerance || rgba[6] > tolerance)
      && (rgba[8] > tolerance || rgba[9] > tolerance || rgba[10] > tolerance)
      && (rgba[12] > tolerance || rgba[13] > tolerance || rgba[14] > tolerance)) {
      // The dot is too bright to be a black dot.
      return false;
    }
  }
  return true;
}

function createFrame(width, height, ts = 0) {
  let duration = 33333;  // 30fps
  let text = ts.toString();
  let cnv = new OffscreenCanvas(width, height);
  var ctx = cnv.getContext('2d');
  fourColorsFrame(ctx, width, height, text);
  return new VideoFrame(cnv, { timestamp: ts, duration });
}

function createDottedFrame(width, height, dots, ts) {
  if (ts === undefined)
    ts = dots;
  let duration = 33333;  // 30fps
  let text = ts.toString();
  let cnv = new OffscreenCanvas(width, height);
  var ctx = cnv.getContext('2d');
  fourColorsFrame(ctx, width, height, text);
  putBlackDots(ctx, width, height, dots);
  return new VideoFrame(cnv, { timestamp: ts, duration });
}

function createVideoEncoder(t, callbacks) {
  return new VideoEncoder({
    output(chunk, metadata) {
      if (callbacks && callbacks.output) {
        t.step(() => callbacks.output(chunk, metadata));
      } else {
        t.unreached_func('unexpected output()');
      }
    },
    error(e) {
      if (callbacks && callbacks.error) {
        t.step(() => callbacks.error(e));
      } else {
        t.unreached_func('unexpected error()');
      }
    }
  });
}
