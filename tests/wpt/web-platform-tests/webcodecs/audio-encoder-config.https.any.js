// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

const invalidConfigs = [
  {
    comment: 'Emtpy codec',
    config: {codec: ''},
  },
  {
    comment: 'Unrecognized codec',
    config: {codec: 'bogus'},
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
    config:
        {codec: 'opus', sampleRate: 8000, numberOfChannels: 2, bitrate: 6e9},
  },
  {
    comment: 'Opus complexity too big',
    config: {
      codec: 'opus',
      sampleRate: 8000,
      numberOfChannels: 2,
      opus: {
        complexity: 11,
      },
    },
  },
  {
    comment: 'Opus packetlossperc too big',
    config: {
      codec: 'opus',
      sampleRate: 8000,
      numberOfChannels: 2,
      opus: {
        packetlossperc: 101,
      },
    },
  },
  {
    comment: 'Opus frame duration too small',
    config: {
      codec: 'opus',
      sampleRate: 8000,
      numberOfChannels: 2,
      opus: {
        frameDuration: 0,
      },
    },
  },
  {
    comment: 'Opus frame duration too big',
    config: {
      codec: 'opus',
      sampleRate: 8000,
      numberOfChannels: 2,
      opus: {
        frameDuration: 122500,
      },
    },
  },
  {
    comment: 'Invalid Opus frameDuration',
    config: {
      codec: 'opus',
      sampleRate: 8000,
      numberOfChannels: 2,
      opus: {
        frameDuration: 2501,
      },
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
  {
    codec: 'opus',
    sampleRate: 48000,
    numberOfChannels: 2,
    opus: {
      complexity: 5,
      frameDuration: 20000,
      packetlossperc: 10,
      useinbandfec: true,
    },
  },
  {
    codec: 'opus',
    sampleRate: 48000,
    numberOfChannels: 2,
    opus: {
      format: 'opus',
      complexity: 10,
      frameDuration: 60000,
      packetlossperc: 20,  // Irrelevant without useinbandfec, but still valid.
      usedtx: true,
      bogus: 456,
    },
  },
  {
    codec: 'opus',
    sampleRate: 48000,
    numberOfChannels: 2,
    opus: {},  // Use default values.
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

    if (config.opus) {
      let opus_config = config.opus;
      let new_opus_config = new_config.opus;

      assert_equals(new_opus_config.format, opus_config.format ?? 'opus');
      assert_equals(
          new_opus_config.frameDuration, opus_config.frameDuration ?? 20000);
      assert_equals(
          new_opus_config.packetlossperc, opus_config.packetlossperc ?? 0);
      assert_equals(
          new_opus_config.useinbandfec, opus_config.useinbandfec ?? false);
      assert_equals(new_opus_config.usedtx, opus_config.usedtx ?? false);
      assert_false(new_opus_config.hasOwnProperty('bogus'));

      if (opus_config.complexity) {
        assert_equals(new_opus_config.complexity, opus_config.complexity);
      } else {
        // Default complexity is 5 for mobile/ARM platforms, and 9 otherwise.
        assert_true(
            new_opus_config.complexity == 5 || new_opus_config.complexity == 9);
      }

    } else {
      assert_false(new_config.hasOwnProperty('opus'));
    }

    assert_false(new_config.hasOwnProperty('bogus'));
  }, "AudioEncoder.isConfigSupported() supports:" + JSON.stringify(config));
});
