// META: global=window
// META: script=/webcodecs/utils.js

// Encode `channels` channels of audio through AudioEncoder and collect all
// output chunks + the first decoderConfig emitted.
async function encode_opus(t, channels) {
  const config = {
    codec: 'opus',
    sampleRate: 48000,
    numberOfChannels: channels,
  };

  const support = await AudioEncoder.isConfigSupported(config);
  assert_true(support.supported,
    `${channels}ch Opus encoding must be supported`);

  let decoder_config = null;
  let chunks = [];

  const encoder = new AudioEncoder({
    output: (chunk, metadata) => {
      if (metadata?.decoderConfig) {
        if (!decoder_config) decoder_config = metadata.decoderConfig;
      }
      chunks.push(chunk);
    },
    error: e => assert_unreached('encoder error: ' + e),
  });

  encoder.configure(config);

  // One second of audio is enough to produce multiple Opus frames.
  const data = make_audio_data(0, channels, 48000, 48000);
  encoder.encode(data);
  await encoder.flush();
  encoder.close();
  data.close();

  return { decoder_config, chunks };
}

// Verify the OpusHead magic bytes ('O','p','u','s','H','e','a','d').
function assert_valid_opushead(description, label) {
  const bytes = new Uint8Array(description);
  assert_greater_than_equal(bytes.length, 19,
    `${label}: description must be at least 19 bytes`);
  assert_equals(bytes[0], 0x4f, `${label}: description[0] == 'O'`);
  assert_equals(bytes[1], 0x70, `${label}: description[1] == 'p'`);
  assert_equals(bytes[2], 0x75, `${label}: description[2] == 'u'`);
  assert_equals(bytes[3], 0x73, `${label}: description[3] == 's'`);
  assert_equals(bytes[4], 0x48, `${label}: description[4] == 'H'`);
  assert_equals(bytes[5], 0x65, `${label}: description[5] == 'e'`);
  assert_equals(bytes[6], 0x61, `${label}: description[6] == 'a'`);
  assert_equals(bytes[7], 0x64, `${label}: description[7] == 'd'`);
}

promise_test(async t => {
  const channels = 16;
  const { decoder_config, chunks } = await encode_opus(t, channels);

  assert_greater_than(chunks.length, 0, 'encoder must produce output');
  assert_not_equals(decoder_config, null,
    'encoder must emit decoderConfig in metadata');
  assert_equals(decoder_config.codec, 'opus');
  assert_equals(decoder_config.numberOfChannels, channels);
  assert_equals(decoder_config.sampleRate, 48000);
  assert_not_equals(decoder_config.description, null,
    'decoderConfig.description must be present for >2ch Opus');

  assert_valid_opushead(decoder_config.description, '16ch');

  // channel count must be in the description (byte 9, 0-indexed)
  const bytes = new Uint8Array(decoder_config.description);
  assert_equals(bytes[9], channels,
    'OpusHead channel count byte must match numberOfChannels');
}, 'Encoding 16ch Opus produces a valid OpusHead decoderConfig description.');

promise_test(async t => {
  const channels = 16;
  const { decoder_config, chunks } = await encode_opus(t, channels);

  assert_not_equals(decoder_config, null, 'need decoderConfig to test decode');
  assert_greater_than(chunks.length, 0, 'need chunks to decode');

  let decoded_frames = [];
  const decoder = new AudioDecoder({
    output: frame => { decoded_frames.push(frame); },
    error: e => assert_unreached('decoder error: ' + e),
  });

  decoder.configure({
    codec: 'opus',
    sampleRate: 48000,
    numberOfChannels: channels,
    description: decoder_config.description,
  });

  for (const chunk of chunks) {
    decoder.decode(chunk);
  }
  await decoder.flush();
  decoder.close();

  assert_greater_than(decoded_frames.length, 0, 'decoder must produce output');
  for (const frame of decoded_frames) {
    assert_equals(frame.numberOfChannels, channels,
      'decoded frame must have ' + channels + ' channels');
    frame.close();
  }
}, 'Encoding then decoding 16ch Opus produces frames with 16 channels.');

promise_test(async t => {
  const config = {
    codec: 'opus',
    sampleRate: 48000,
    numberOfChannels: 16,
    // no description
  };

  // configure() queues work; the codec-specific failure surfaces asynchronously
  // through the error callback.
  const error = await new Promise(resolve => {
    const decoder = new AudioDecoder({
      output: () => {},
      error: e => resolve(e),
    });
    decoder.configure(config);
  });

  assert_equals(error.name, 'NotSupportedError',
    '>2ch Opus without description must fire NotSupportedError');
}, 'Configuring AudioDecoder with 16ch Opus without description must fail.');
