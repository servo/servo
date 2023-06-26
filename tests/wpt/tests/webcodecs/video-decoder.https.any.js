// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

const invalidConfigs = [
  {
    comment: 'Empty codec',
    config: {codec: ''},
  },
  {
    comment: 'Unrecognized codec',
    config: {codec: 'bogus'},
  },
  {
    comment: 'Audio codec',
    config: {codec: 'vorbis'},
  },
  {
    comment: 'Ambiguous codec',
    config: {codec: 'vp9'},
  },
  {
    comment: 'Codec with MIME type',
    config: {codec: 'video/webm; codecs="vp8"'},
  },
];  //  invalidConfigs

invalidConfigs.forEach(entry => {
  promise_test(
      t => {
        return promise_rejects_js(
            t, TypeError, VideoDecoder.isConfigSupported(entry.config));
      },
      'Test that VideoDecoder.isConfigSupported() rejects invalid config:' +
          entry.comment);
});

invalidConfigs.forEach(entry => {
  async_test(
      t => {
        let codec = new VideoDecoder(getDefaultCodecInit(t));
        assert_throws_js(TypeError, () => {
          codec.configure(entry.config);
        });
        t.done();
      },
      'Test that VideoDecoder.configure() rejects invalid config:' +
          entry.comment);
});

promise_test(t => {
  // VideoDecoderInit lacks required fields.
  assert_throws_js(TypeError, () => {
    new VideoDecoder({});
  });

  // VideoDecoderInit has required fields.
  let decoder = new VideoDecoder(getDefaultCodecInit(t));

  assert_equals(decoder.state, 'unconfigured');

  decoder.close();

  return endAfterEventLoopTurn();
}, 'Test VideoDecoder construction');
