// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

const detachedArrayBuffer = new ArrayBuffer(4);
var b = detachedArrayBuffer.transferToFixedLength();

const invalidConfigs = [
  {
    comment: 'Missing codec',
    config: {},
  },
  {
    comment: 'Empty codec',
    config: {codec: ''},
  },
  {
    comment: 'Valid codec, detached description',
    config: {codec: 'vp8', description: detachedArrayBuffer},
  },
];  // invalidConfigs

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

const arrayBuffer = new ArrayBuffer(12583);
const arrayBufferView = new DataView(arrayBuffer);

const validButUnsupportedConfigs = [
  {
    comment: 'Unrecognized codec',
    config: {codec: 'bogus'},
  },
  {
    comment: 'Unrecognized codec with dataview description',
    config: {
      codec: '7󠎢ﷺ۹.9',
      description: arrayBufferView,
    },
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
    comment: 'Codec with bad casing',
    config: {codec: 'Vp09.00.10.08'},
  },
  {
    comment: 'Codec with MIME type',
    config: {codec: 'video/webm; codecs="vp8"'},
  },
  {
    comment: 'Possible future H264 codec string',
    config: {codec: 'avc1.FF000b'},
  },
  {
    comment: 'Possible future H264 codec string (level 2.9)',
    config: {codec: 'avc1.4D401D'},
  },
  {
    comment: 'Possible future HEVC codec string',
    config: {codec: 'hvc1.C99.6FFFFFF.L93'},
  },
  {
    comment: 'Possible future VP9 codec string',
    config: {codec: 'vp09.99.99.08'},
  },
  {
    comment: 'Possible future AV1 codec string',
    config: {codec: 'av01.9.99M.08'},
  },
  {
    comment: 'codec with spaces',
    config: {codec: '  vp09.00.10.08  '},
  },
];  //  validButUnsupportedConfigs

validButUnsupportedConfigs.forEach(entry => {
  promise_test(
      t => {
        return VideoDecoder.isConfigSupported(entry.config).then(support => {
          assert_false(support.supported);
        });
      },
      'Test that VideoDecoder.isConfigSupported() doesn\'t support config: ' +
          entry.comment);
});

validButUnsupportedConfigs.forEach(entry => {
  promise_test(
      t => {
        let isErrorCallbackCalled = false;
        let codec = new VideoDecoder({
          output: t.unreached_func('unexpected output'),
          error: t.step_func(e => {
            isErrorCallbackCalled = true;
            assert_true(e instanceof DOMException);
            assert_equals(e.name, 'NotSupportedError');
            assert_equals(codec.state, 'closed', 'state');
          })
        });
        codec.configure(entry.config);
        return codec.flush()
            .then(t.unreached_func('flush succeeded unexpectedly'))
            .catch(t.step_func(e => {
              assert_true(isErrorCallbackCalled, "isErrorCallbackCalled");
              assert_true(e instanceof DOMException);
              assert_equals(e.name, 'NotSupportedError');
              assert_equals(codec.state, 'closed', 'state');
            }));
      },
      'Test that VideoDecoder.configure() doesn\'t support config: ' +
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

const validConfigs = [
  {
    comment: 'variant 1 of h264 codec string',
    config: {codec: 'avc3.42001E'},
  },
  {
    comment: 'variant 2 of h264 codec string',
    config: {codec: 'avc1.42001E'},
  },
];  // validConfigs

validConfigs.forEach(entry => {
  promise_test(async t => {
    try {
      await VideoDecoder.isConfigSupported(entry.config);
      var decoder = new VideoDecoder(getDefaultCodecInit(t));
      // Something that works with all codecs:
      entry.config.width = 1280;
      entry.config.height = 720;
      decoder.configure(entry.config);
      return decoder
        .flush()
        .then(
          t.step_func(e => {
            assert_equals(decoder.state, 'configured', 'codec is configured');
          })
        )
        .catch(t.unreached_func('flush succeeded unexpectedly'));
    } catch (e) {
      assert_true(false, entry.comment + ' should not throw');
    }
  }, 'Test that VideoDecoder.isConfigSupported() accepts config:' + entry.comment);
});
