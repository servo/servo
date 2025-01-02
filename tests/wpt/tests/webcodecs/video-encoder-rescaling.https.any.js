// META: global=window,dedicatedworker
// META: variant=?av1
// META: variant=?vp8
// META: variant=?vp9_p0
// META: variant=?h264_avc
// META: variant=?h264_annexb

let BASECONFIG = null;
promise_setup(async () => {
  const config = {
    '?av1': { codec: 'av01.0.04M.08' },
    '?vp8': { codec: 'vp8' },
    '?vp9_p0': { codec: 'vp09.00.10.08' },
    '?h264_avc': { codec: 'avc1.42001E', avc: { format: 'avc' } },
    '?h264_annexb': { codec: 'avc1.42001E', avc: { format: 'annexb' } },
  }[location.search];
  BASECONFIG = config;
  BASECONFIG.framerate = 30;
  BASECONFIG.bitrate = 3000000;
});

function scaleFrame(oneFrame, scaleSize) {
  const { w: width, h: height } = scaleSize;
  return new Promise(async (resolve, reject) => {
    let encodedResult;
    const encoder = new VideoEncoder({
      output: (chunk, metadata) => {
        encodedResult = { chunk, metadata };
      },
      error: (error) => {
        reject(error);
      },
    });

    const encoderConfig = {
      ...BASECONFIG,
      width,
      height,
    };
    encoder.configure(encoderConfig);

    encoder.encode(oneFrame);
    await encoder.flush();

    let decodedResult;
    const decoder = new VideoDecoder({
      output(frame) {
        decodedResult = frame;
      },
      error: (error) => {
        reject(error);
      },
    });

    decoder.configure(encodedResult.metadata.decoderConfig);
    decoder.decode(encodedResult.chunk);
    await decoder.flush();

    encoder.close();
    decoder.close();

    resolve(decodedResult);
  });
}

// This function determines which quadrant of a rectangle (width * height)
// a point (x, y) falls into, and returns the corresponding color for that
// quadrant. The rectangle is divided into four quadrants:
//    <        w        >
//  ^ +--------+--------+
//    | (0, 0) | (1, 0) |
//  h +--------+--------+
//    | (0, 1) | (1, 1) |
//  v +--------+--------+
//
// The colors array must contain at least four colors, each corresponding
// to one of the quadrants:
// - colors[0] : top-left (0, 0)
// - colors[1] : top-right (1, 0)
// - colors[2] : bottom-left (0, 1)
// - colors[3] : bottom-right (1, 1)
function getColor(x, y, width, height, colors, channel) {
  // Determine which quadrant (x, y) belongs to.
  const xIndex = x * 2 >= width ? 1 : 0;
  const yIndex = y * 2 >= height ? 1 : 0;

  const index = yIndex * 2 + xIndex;
  return colors[index][channel];
}


// All channel paramaters are arrays with the index being the channel
// channelOffset: The offset for each channel in allocated data array.
// channelWidth: The width of ecah channel in pixels
// channelPlaneWidths: the width of the channel used to calculate the image's memory size.
//   For interleaved data, only the first width is set to the width of the full data in bytes; see RGBX for an example.
// channelStrides: The stride (in bytes) for each channel.
// channelSteps: The step size in bytes to move from one pixel to the next horizontally within the same row
// channelHeights: The height (in bytes) for each channel.
// channelFourColor: The four colors encoded in the color format of the channels
//
function createImageData({ channelOffsets, channelWidths, channelPlaneWidths, channelStrides, channelSteps, channelHeights, channelFourColors }) {
  let memSize = 0;
  for (let chan = 0; chan < 3; chan++) {
    memSize += channelHeights[chan] * channelPlaneWidths[chan];
  }
  let data = new Uint8Array(memSize);
  for (let chan = 0; chan < 3; chan++) {
    for (let y = 0; y < channelHeights[chan]; y++) {
      for (let x = 0; x < channelWidths[chan]; x++) {
        data[channelOffsets[chan] + Math.floor(channelStrides[chan] * y) + Math.floor(channelSteps[chan] * x)] =
          getColor(x, y, channelWidths[chan], channelHeights[chan], channelFourColors, chan);
      }
    }
  }
  return data;
}

function testImageData(data, { channelOffsets, channelWidths, channelStrides, channelSteps, channelHeights, channelFourColors }) {
  let err = 0.;
  for (let chan = 0; chan < 3; chan++) {
    for (let y = 0; y < channelHeights[chan]; y++) {
      for (let x = 0; x < channelWidths[chan]; x++) {
        const curdata = data[channelOffsets[chan] + Math.floor(channelStrides[chan] *  y) + Math.floor(channelSteps[chan] * x)];
        const diff = curdata - getColor(x, y, channelWidths[chan], channelHeights[chan], channelFourColors, chan);
        err += Math.abs(diff);
      }
    }
  }
  return err / data.length / 3 / 255 * 4;
}

function rgb2yuv(rgb) {
  let y = rgb[0] * .299000 + rgb[1] * .587000 + rgb[2] * .114000
  let u = rgb[0] * -.168736 + rgb[1] * -.331264 + rgb[2] * .500000 + 128
  let v = rgb[0] * .500000 + rgb[1] * -.418688 + rgb[2] * -.081312 + 128

  y = Math.floor(y);
  u = Math.floor(u);
  v = Math.floor(v);
  return [
    y, u, v
  ]
}

function createChannelParameters(channelParams, x, y) {
  return {
    channelOffsets: channelParams.channelOffsetsConstant.map(
      (cont, index) => cont + channelParams.channelOffsetsSize[index] *
        x * y),
    channelWidths: channelParams.channelWidths.map((width) => Math.floor(width * x)),
    channelPlaneWidths: channelParams.channelPlaneWidths.map((width) => Math.floor(width * x)),
    channelStrides: channelParams.channelStrides.map((width) => Math.floor(width * x)),
    channelSteps: channelParams.channelSteps.map((height) => height),
    channelHeights: channelParams.channelHeights.map((height) => Math.floor(height * y)),
    channelFourColors: channelParams.channelFourColors
  }
}


const scaleTests = [
  { from: { w: 64, h: 64 }, to: { w: 128, h: 128 } }, // Factor 2
  { from: { w: 128, h: 128 }, to: { w: 128, h: 128 } }, // Factor 1
  { from: { w: 128, h: 128 }, to: { w: 64, h: 64 } }, // Factor 0.5
  { from: { w: 32, h: 32 }, to: { w: 96, h: 96 } }, // Factor 3
  { from: { w: 192, h: 192 }, to: { w: 64, h: 64 } }, // Factor 1/3
  { from: { w: 64, h: 32 }, to: { w: 128, h: 64 } }, // Factor 2
  { from: { w: 128, h: 256 }, to: { w: 64, h: 128 } }, // Factor 0.5
  { from: { w: 64, h: 64 }, to: { w: 128, h: 192 } }, // Factor 2 (w) and 3 (h)
  { from: { w: 128, h: 192 }, to: { w: 64, h: 64 } }, // Factor 0.5 (w) and 1/3 (h)
]
const fourColors = [[255, 255, 0], [255, 0, 0], [0, 255, 0], [0, 0, 255]];
const pixelFormatChannelParameters = [
  { // RGBX
    channelOffsetsConstant: [0, 1, 2],
    channelOffsetsSize: [0, 0, 0],
    channelPlaneWidths: [4, 0, 0], // only used for allocation
    channelWidths: [1, 1, 1],
    channelStrides: [4, 4, 4], // scaled by width
    channelSteps: [4, 4, 4],
    channelHeights: [1, 1, 1],  // scaled by height
    channelFourColors: fourColors.map((col) => col), // just clone,
    format: 'RGBX'
  },
  { // I420
    channelOffsetsConstant: [0, 0, 0],
    channelOffsetsSize: [0, 1, 1.25],
    channelPlaneWidths: [1, 0.5, 0.5],
    channelWidths: [1, 0.5, 0.5],
    channelStrides: [1, 0.5, 0.5], // scaled by width
    channelSteps: [1, 1, 1],
    channelHeights: [1, 0.5, 0.5],  // scaled by height
    channelFourColors: fourColors.map((col) => rgb2yuv(col)), // just clone
    format: 'I420'
  }
]

for (const scale of scaleTests) {
  for (const channelParams of pixelFormatChannelParameters) {
    promise_test(async t => {
      const inputChannelParameters = createChannelParameters(channelParams, scale.from.w, scale.from.h);
      const inputData = createImageData(inputChannelParameters);
      const inputFrame = new VideoFrame(inputData, {
        timestamp: 0,
        displayWidth: scale.from.w,
        displayHeight: scale.from.h,
        codedWidth: scale.from.w,
        codedHeight: scale.from.h,
        format: channelParams.format
      });
      const outputFrame = await scaleFrame(inputFrame, scale.to);
      const outputArrayBuffer = new Uint8Array(outputFrame.allocationSize({ format: 'RGBX' }));
      const layout = await outputFrame.copyTo(outputArrayBuffer, { format: 'RGBX' });
      const stride = layout[0].stride
      const offset = layout[0].offset

      const error = testImageData(outputArrayBuffer, {
        channelOffsets: [offset, offset + 1, offset + 2],
        channelWidths: [outputFrame.codedWidth, outputFrame.codedWidth, outputFrame.codedWidth],
        channelStrides: [stride, stride, stride],
        channelSteps: [4, 4, 4],
        channelHeights: [outputFrame.codedHeight, outputFrame.codedHeight, outputFrame.codedHeight],
        channelFourColors: fourColors.map((col) => col)
      });
      outputFrame.close();
      assert_approx_equals(error, 0, 0.05, 'Scaled Image differs too much! Scaling from '
        + scale.from.w + ' x ' + scale.from.h
        + ' to '
        + scale.to.w + ' x ' + scale.to.h
        + ' Format:' +
        channelParams.format
      );
    }, 'Scaling Image in Encoding from '
    + scale.from.w + ' x ' + scale.from.h
    + ' to '
    + scale.to.w + ' x ' + scale.to.h
    + ' Format: ' +
    channelParams.format);
  }
}
