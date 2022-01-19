// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

const invalidConfigs = [
  {
    comment: 'Emtpy codec',
    config: { codec: '' },
  },
  {
    comment: 'Unrecognized codec',
    config: { codec: 'bogus' },
  },
  {
    comment: 'Sample rate is too small',
    config: {
      codec: 'opus',
      sampleRate: 100,
      numberOfChannels: 2,
    },
  },
  {
    comment: 'Sample rate is too large',
    config: {
      codec: 'opus',
      sampleRate: 1e6,
      numberOfChannels: 2,
    },
  },
  {
    comment: 'Too few channels',
    config: {
      codec: 'opus',
      sampleRate: 8000,
      numberOfChannels: 0,
    },
  },
  {
    comment: 'Way too many channels',
    config: {
      codec: 'opus',
      sampleRate: 8000,
      numberOfChannels: 100,
      bitrate: 128000
    },
  },
  {
    comment: 'Bit rate too big',
    config: {
      codec: 'opus',
      sampleRate: 8000,
      numberOfChannels: 2,
      bitrate: 6e9
    },
  },
];

invalidConfigs.forEach(entry => {
  promise_test(t => {
    return promise_rejects_js(t, TypeError, AudioEncoder.isConfigSupported(entry.config));
  }, 'Test that AudioEncoder.isConfigSupported() rejects invalid config:' + entry.comment);
});

const validButUnsupportedConfigs = [
  {
    comment: 'Too many channels',
    config: {
      codec: 'opus',
      sampleRate: 48000,
      numberOfChannels: 30,
    },
  },
  {
    comment: 'Bitrate is too low',
    config: {
      codec: 'opus',
      sampleRate: 48000,
      numberOfChannels: 2,
      bitrate: 1
    },
  }
];

validButUnsupportedConfigs.forEach(entry => {
  promise_test(async t => {
    let support = await AudioEncoder.isConfigSupported(entry.config);
    assert_false(support.supported);

    let config = support.config;
    assert_equals(config.codec, entry.config.codec);
    assert_equals(config.sampleRate, entry.config.sampleRate);
    assert_equals(config.numberOfChannels, entry.config.numberOfChannels);

  }, "Test that AudioEncoder.isConfigSupported() doesn't support config:" + entry.comment);
});

const validConfigs = [
  {
    codec: 'opus',
    sampleRate: 8000,
    numberOfChannels: 1,
  },
  {
    codec: 'opus',
    sampleRate: 48000,
    numberOfChannels: 2,
  },
  {
    codec: 'opus',
    sampleRate: 48000,
    numberOfChannels: 2,
    bitrate: 128000,
    bogus: 123
  },
];

validConfigs.forEach(config => {
  promise_test(async t => {
    let support = await AudioEncoder.isConfigSupported(config);
    assert_true(support.supported);

    let new_config = support.config;
    assert_equals(new_config.codec, config.codec);
    assert_equals(new_config.sampleRate, config.sampleRate);
    assert_equals(new_config.numberOfChannels, config.numberOfChannels);
    if (config.bitrate)
      assert_equals(new_config.bitrate, config.bitrate);
    assert_false(new_config.hasOwnProperty('bogus'));
  }, "AudioEncoder.isConfigSupported() supports:" + JSON.stringify(config));
});
