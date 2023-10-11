// META: global=window,dedicatedworker
// META: script=/common/media.js
// META: script=/webcodecs/utils.js
// META: script=/webcodecs/video-encoder-utils.js
// META: variant=?baseline
// META: variant=?main
// META: variant=?high

promise_test(async t => {
  const codecString = {
    '?baseline': 'avc1.42001e',
    '?main': 'avc1.4d001e',
    '?high': 'avc1.64001e',
  }[location.search];

  let encoderConfig = {
    codec: codecString,
    width: 640,
    height: 480,
    displayWidth: 800,
    displayHeight: 600,
    avc: {format: 'avc'},  // AVC makes it easy to check the profile.
  };

  let supported = false;
  try {
    const support = await VideoEncoder.isConfigSupported(encoderConfig);
    supported = support.supported;
  } catch (e) {
  }
  assert_implements_optional(
      supported, `H264 ${location.search.substring(1)} profile unsupported`);

  let description = null;
  let codecInit = {
    error: t.unreached_func('Unexpected encoding error'),
    output: (_, metadata) => {
      assert_not_equals(metadata, null);
      if (metadata.decoderConfig)
        description = metadata.decoderConfig.description;
    },
  };

  let encoder = new VideoEncoder(codecInit);
  encoder.configure(encoderConfig);

  let frame1 = createFrame(640, 480, 0);
  let frame2 = createFrame(640, 480, 33333);
  t.add_cleanup(() => {
    frame1.close();
    frame2.close();
  });

  encoder.encode(frame1);
  encoder.encode(frame2);

  await encoder.flush();

  assert_not_equals(description, null);
  assert_not_equals(description.length, 0);

  // Profile is a hex code at xx in a codec string of form "avc1.xxyyzz".
  let expectedProfileIndication = parseInt(codecString.substring(5, 7), 16);

  // See AVCDecoderConfigurationRecord in ISO/IEC 14496-15 for details.
  // https://www.w3.org/TR/webcodecs-avc-codec-registration/#dom-avcbitstreamformat-avc
  let profileIndication = new Uint8Array(description)[1];
  assert_equals(profileIndication, expectedProfileIndication);
}, 'Test that encoding with a specific H264 profile actually produces that profile.');
