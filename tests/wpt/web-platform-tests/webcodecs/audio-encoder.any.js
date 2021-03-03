// META: global=window
// META: script=/webcodecs/utils.js

function make_audio_frame(timestamp, channels, sampleRate, length) {
  let buffer = new AudioBuffer({
    length: length,
    numberOfChannels: channels,
    sampleRate: sampleRate
  });

  for (var channel = 0; channel < buffer.numberOfChannels; channel++) {
    // This gives us the actual array that contains the data
    var array = buffer.getChannelData(channel);
    let hz = 100 + channel * 50; // sound frequency
    for (var i = 0; i < array.length; i++) {
      let t = (i / sampleRate) * hz * (Math.PI * 2);
      array[i] = Math.sin(t);
    }
  }

  return new AudioFrame({
    timestamp: timestamp,
    buffer: buffer
  });
}

// Merge all audio buffers into a new big one with all the data.
function join_buffers(buffers) {
  assert_greater_than_equal(buffers.length, 0);
  let total_length = 0;
  let base_buffer = buffers[0];
  for (const buffer of buffers) {
    assert_not_equals(buffer, null);
    assert_equals(buffer.sampleRate, base_buffer.sampleRate);
    assert_equals(buffer.numberOfChannels, base_buffer.numberOfChannels);
    total_length += buffer.length;
  }

  let result = new AudioBuffer({
    length: total_length,
    numberOfChannels: base_buffer.numberOfChannels,
    sampleRate: base_buffer.sampleRate
  });

  for (let i = 0; i < base_buffer.numberOfChannels; i++) {
    let channel = result.getChannelData(i);
    let position = 0;
    for (const buffer of buffers) {
      channel.set(buffer.getChannelData(i), position);
      position += buffer.length;
    }
    assert_equals(position, total_length);
  }

  return result;
}

function clone_frame(frame) {
  return new AudioFrame({
    timestamp: frame.timestamp,
    buffer: join_buffers([frame.buffer])
  });
}

promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 1;
  let frame_count = 10;
  let outputs = [];
  let init = {
    error: e => {
      assert_unreached("error: " + e);
    },
    output: chunk => {
      outputs.push(chunk);
    }
  };

  let encoder = new AudioEncoder(init);

  assert_equals(encoder.state, "unconfigured");
  let config = {
    codec: 'opus',
    sampleRate: sample_rate,
    numberOfChannels: 2,
    bitrate: 256000 //256kbit
  };

  encoder.configure(config);

  let timestamp_us = 0;
  let frame_duration_s = total_duration_s / frame_count;
  let frame_length = frame_duration_s * config.sampleRate;
  for (let i = 0; i < frame_count; i++) {
    let frame = make_audio_frame(timestamp_us, config.numberOfChannels,
      config.sampleRate, frame_length);
    encoder.encode(frame);
    timestamp_us += frame_duration_s * 1_000_000;
  }
  await encoder.flush();
  encoder.close();
  assert_greater_than_equal(outputs.length, frame_count);
  assert_equals(outputs[0].timestamp, 0, "first chunk timestamp");
  for (chunk of outputs) {
    assert_greater_than(chunk.data.byteLength, 0);
    assert_greater_than(timestamp_us, chunk.timestamp);
  }
}, 'Simple audio encoding');


async function checkEncodingError(config, good_frames, bad_frame) {
  let error = null;
  let outputs = 0;
  let init = {
    error: e => {
      error = e;
    },
    output: chunk => {
      outputs++;
    }
  };

  let encoder = new AudioEncoder(init);

  encoder.configure(config);
  for (let frame of good_frames) {
    encoder.encode(frame);
  }
  await encoder.flush();

  let txt_config = "sampleRate: " + config.sampleRate
                 + " numberOfChannels: " + config.numberOfChannels;
  assert_equals(error, null, txt_config);
  assert_greater_than(outputs, 0);
  encoder.encode(bad_frame);
  await encoder.flush().catch(() => {});
  assert_not_equals(error, null, txt_config);
}

function channelNumberVariationTests() {
  let sample_rate = 48000;
  for (let channels = 1; channels < 12; channels++) {
    let config = {
      codec: 'opus',
      sampleRate: sample_rate,
      numberOfChannels: channels,
      bitrate: 128000
    };

    let ts = 0;
    let length = sample_rate / 10;
    let frame1 = make_audio_frame(ts, channels, sample_rate, length);

    ts += Math.floor(frame1.buffer.duration / 1000000);
    let frame2 = make_audio_frame(ts, channels, sample_rate, length);
    ts += Math.floor(frame2.buffer.duration / 1000000);

    let bad_frame = make_audio_frame(ts, channels + 1, sample_rate, length);
    promise_test(async t =>
      checkEncodingError(config, [frame1, frame2], bad_frame),
      "Channel number variation: " + channels);
  }
}
channelNumberVariationTests();

function sampleRateVariationTests() {
  let channels = 1
  for (let sample_rate = 3000; sample_rate < 96000; sample_rate += 10000) {
    let config = {
      codec: 'opus',
      sampleRate: sample_rate,
      numberOfChannels: channels,
      bitrate: 128000
    };

    let ts = 0;
    let length = sample_rate / 10;
    let frame1 = make_audio_frame(ts, channels, sample_rate, length);

    ts += Math.floor(frame1.buffer.duration / 1000000);
    let frame2 = make_audio_frame(ts, channels, sample_rate, length);
    ts += Math.floor(frame2.buffer.duration / 1000000);

    let bad_frame = make_audio_frame(ts, channels, sample_rate + 333, length);
    promise_test(async t =>
      checkEncodingError(config, [frame1, frame2], bad_frame),
      "Sample rate variation: " + sample_rate);
  }
}
sampleRateVariationTests();

promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 1;
  let frame_count = 10;
  let input_frames = [];
  let output_frames = [];

  let decoder_init = {
    error: t.unreached_func("Decode error"),
    output: frame => {
      output_frames.push(frame);
    }
  };
  let decoder = new AudioDecoder(decoder_init);

  let encoder_init = {
    error: t.unreached_func("Encoder error"),
    output: (chunk, config) => {
      if (config)
        decoder.configure(config);
      decoder.decode(chunk);
    }
  };
  let encoder = new AudioEncoder(encoder_init);

  let config = {
    codec: 'opus',
    sampleRate: sample_rate,
    numberOfChannels: 2,
    bitrate: 256000, //256kbit
  };
  encoder.configure(config);

  let timestamp_us = 0;
  const frame_duration_s = total_duration_s / frame_count;
  const frame_length = frame_duration_s * config.sampleRate;
  for (let i = 0; i < frame_count; i++) {
    let frame = make_audio_frame(timestamp_us, config.numberOfChannels,
      config.sampleRate, frame_length);
    input_frames.push(clone_frame(frame));
    encoder.encode(frame);
    timestamp_us += frame_duration_s * 1_000_000;
  }
  await encoder.flush();
  encoder.close();
  await decoder.flush();
  decoder.close();


  let total_input = join_buffers(input_frames.map(f => f.buffer));
  let total_output = join_buffers(output_frames.map(f => f.buffer));
  assert_equals(total_output.numberOfChannels, 2);
  assert_equals(total_output.sampleRate, sample_rate);

  // Output can be slightly longer that the input due to padding
  assert_greater_than_equal(total_output.length, total_input.length);
  assert_greater_than_equal(total_output.duration, total_duration_s);
  assert_approx_equals(total_output.duration, total_duration_s, 0.1);

  // Compare waveform before and after encoding
  for (let channel = 0; channel < total_input.numberOfChannels; channel++) {
    let input_data = total_input.getChannelData(channel);
    let output_data = total_output.getChannelData(channel);
    for (let i = 0; i < total_input.length; i += 10) {
      // Checking only every 10th sample to save test time in slow
      // configurations like MSAN etc.
      assert_approx_equals(input_data[i], output_data[i], 0.5,
        "Difference between input and output is too large."
        + " index: " + i
        + " input: " + input_data[i]
        + " output: " + output_data[i]);
    }
  }

}, 'Encoding and decoding');

promise_test(async t => {
  let output_count = 0;
  let encoder_config = {
    codec: 'opus',
    sampleRate: 24000,
    numberOfChannels: 1,
    bitrate: 96000
  };
  let decoder_config = null;

  let init = {
    error: t.unreached_func("Encoder error"),
    output: (chunk, config) => {
      // Only the first invocation of the output callback is supposed to have
      // a |config| in it.
      output_count++;
      if (output_count == 1) {
        assert_equals(typeof config, "object");
        decoder_config = config;
      } else {
        assert_equals(config, undefined);
      }
    }
  };

  let encoder = new AudioEncoder(init);
  encoder.configure(encoder_config);

  let long_frame = make_audio_frame(0, encoder_config.numberOfChannels,
    encoder_config.sampleRate, encoder_config.sampleRate);
  encoder.encode(clone_frame(long_frame));
  await encoder.flush();

  // Long frame produced more than one output, and we've got decoder_config
  assert_greater_than(output_count, 1);
  assert_not_equals(decoder_config, null);
  assert_equals(decoder_config.codec, encoder_config.codec);
  assert_equals(decoder_config.sampleRate, encoder_config.sampleRate);
  assert_equals(decoder_config.numberOfChannels, encoder_config.numberOfChannels);

  // Check that description start with 'Opus'
  let extra_data = new Uint8Array(decoder_config.description);
  assert_equals(extra_data[0], 0x4f);
  assert_equals(extra_data[1], 0x70);
  assert_equals(extra_data[2], 0x75);
  assert_equals(extra_data[3], 0x73);

  decoder_config = null;
  output_count = 0;
  encoder_config.bitrate = 256000;
  encoder.configure(encoder_config);
  encoder.encode(clone_frame(long_frame));
  await encoder.flush();

  // After reconfiguring encoder should produce decoder config again
  assert_greater_than(output_count, 1);
  assert_not_equals(decoder_config, null);
  assert_not_equals(decoder_config.description, null);
  encoder.close();
}, "Emit decoder config and extra data.");