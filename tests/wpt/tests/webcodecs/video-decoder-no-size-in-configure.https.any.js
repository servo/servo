// META: global=window,dedicatedworker
// META: script=/webcodecs/video-encoder-utils.js
// META: variant=?av1
// META: variant=?vp8
// META: variant=?vp9_p0
// META: variant=?h264_avc
// META: variant=?h264_annexb

var CODEC = null;
promise_setup(async () => {
  CODEC = {
    '?av1': { codec: 'av01.0.04M.08' },
    '?vp8': { codec: 'vp8' },
    '?vp9_p0': { codec: 'vp09.00.10.08' },
    '?h264_avc': { codec: 'avc1.42001E', avc: { format: 'avc' } },
    '?h264_annexb': { codec: 'avc1.42001E', avc: { format: 'annexb' } },
  }[location.search];
});

promise_test(async t => {
  let encoderConfig = {
    ...CODEC,
    width: 320,
    height: 240,
  };

  const encoderSupport = await VideoEncoder.isConfigSupported(encoderConfig);
  assert_implements_optional(encoderSupport.supported,  `${encoderConfig.codec} encoder is unsupported`);

  let encodedResult;
  const encoder = new VideoEncoder({
    output: (chunk, metadata) => {
      encodedResult = { chunk, metadata };
    },
    error: e => {
      t.unreached_func('Unexpected encoding error: ' + e);
    },
  });

  encoderConfig.framerate = 30;
  encoderConfig.bitrate = 3000000;
  encoder.configure(encoderConfig);

  let frame = createFrame(encoderConfig.width, encoderConfig.height, 0);
  encoder.encode(frame);
  frame.close();

  await encoder.flush();
  encoder.close();

  let decoderConfig = encodedResult.metadata.decoderConfig;
  delete decoderConfig.codedWidth;
  delete decoderConfig.codedHeight;
  delete decoderConfig.displayAspectWidth;
  delete decoderConfig.displayAspectHeight;

  const decoderSupport = await VideoDecoder.isConfigSupported(decoderConfig);
  assert_implements_optional(decoderSupport.supported,  `${decoderConfig.codec} decoder is unsupported`);

  let decodedResult;
  const decoder = new VideoDecoder({
    output: frame => {
      decodedResult = frame;
    },
    error: e => {
      t.unreached_func('Unexpected decoding error: ' + e);
    },
  });


  decoder.configure(decoderConfig);
  decoder.decode(encodedResult.chunk);
  await decoder.flush();

  assert_equals(decodedResult.codedWidth, encoderConfig.width, 'decoded frame width');
  assert_equals(decodedResult.codedHeight, encoderConfig.height, 'decoded frame height');
}, 'Test configure() without setting width and height');
