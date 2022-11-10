// META: global=window
// META: script=/webcodecs/utils.js

// Merge all audio buffers into a new big one with all the data.
function join_audio_data(audio_data_array) {
  assert_greater_than_equal(audio_data_array.length, 0);
  let total_frames = 0;
  let base_buffer = audio_data_array[0];
  for (const data of audio_data_array) {
    assert_not_equals(data, null);
    assert_equals(data.sampleRate, base_buffer.sampleRate);
    assert_equals(data.numberOfChannels, base_buffer.numberOfChannels);
    assert_equals(data.format, base_buffer.format);
    total_frames += data.numberOfFrames;
  }

  assert_true(base_buffer.format == 'f32' || base_buffer.format == 'f32-planar');

  if (base_buffer.format == 'f32')
    return join_interleaved_data(audio_data_array, total_frames);

  // The format is 'FLTP'.
  return join_planar_data(audio_data_array, total_frames);
}

function join_interleaved_data(audio_data_array, total_frames) {
  let base_data =  audio_data_array[0];
  let channels = base_data.numberOfChannels;
  let total_samples = total_frames * channels;

  let result = new Float32Array(total_samples);

  let copy_dest = new Float32Array(base_data.numberOfFrames * channels);

  // Copy all the interleaved data.
  let position = 0;
  for (const data of audio_data_array) {
    let samples = data.numberOfFrames * channels;
    if (copy_dest.length < samples)
      copy_dest = new Float32Array(samples);

    data.copyTo(copy_dest, {planeIndex: 0});
    result.set(copy_dest, position);
    position += samples;
  }

  assert_equals(position, total_samples);

  return result;
}

function join_planar_data(audio_data_array, total_frames) {
  let base_frames = audio_data_array[0].numberOfFrames;
  let channels = audio_data_array[0].numberOfChannels;
  let result = new Float32Array(total_frames*channels);
  let copyDest = new Float32Array(base_frames);

  // Merge all samples and lay them out according to the FLTP memory layout.
  let position = 0;
  for (let ch = 0; ch < channels; ch++) {
    for (const data of audio_data_array) {
      data.copyTo(copyDest, { planeIndex: ch});
      result.set(copyDest, position);
      position += data.numberOfFrames;
    }
  }
  assert_equals(position, total_frames * channels);

  return result;
}

promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 1;
  let data_count = 10;
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
  let data_duration_s = total_duration_s / data_count;
  let data_length = data_duration_s * config.sampleRate;
  for (let i = 0; i < data_count; i++) {
    let data = make_audio_data(timestamp_us, config.numberOfChannels,
      config.sampleRate, data_length);
    encoder.encode(data);
    data.close();
    timestamp_us += data_duration_s * 1_000_000;
  }
  await encoder.flush();
  encoder.close();
  assert_greater_than_equal(outputs.length, data_count);
  assert_equals(outputs[0].timestamp, 0, "first chunk timestamp");
  let total_encoded_duration = 0
  for (chunk of outputs) {
    assert_greater_than(chunk.byteLength, 0);
    assert_greater_than_equal(timestamp_us, chunk.timestamp);
    assert_greater_than(chunk.duration, 0);
    total_encoded_duration += chunk.duration;
  }

  // The total duration might be padded with silence.
  assert_greater_than_equal(
      total_encoded_duration, total_duration_s * 1_000_000);
}, 'Simple audio encoding');

promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 1;
  let data_count = 10;
  let outputs = [];
  let init = {
    error: e => {
      assert_unreached('error: ' + e);
    },
    output: chunk => {
      outputs.push(chunk);
    }
  };

  let encoder = new AudioEncoder(init);

  assert_equals(encoder.state, 'unconfigured');
  let config = {
    codec: 'opus',
    sampleRate: sample_rate,
    numberOfChannels: 2,
    bitrate: 256000  // 256kbit
  };

  encoder.configure(config);

  let timestamp_us = -10000;
  let data = make_audio_data(
      timestamp_us, config.numberOfChannels, config.sampleRate, 10000);
  encoder.encode(data);
  data.close();
  await encoder.flush();
  encoder.close();
  assert_greater_than_equal(outputs.length, 1);
  assert_equals(outputs[0].timestamp, -10000, 'first chunk timestamp');
  for (chunk of outputs) {
    assert_greater_than(chunk.byteLength, 0);
    assert_greater_than_equal(chunk.timestamp, timestamp_us);
  }
}, 'Encode audio with negative timestamp');

async function checkEncodingError(config, good_data, bad_data) {
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


  let support = await AudioEncoder.isConfigSupported(config);
  assert_true(support.supported)
  config = support.config;

  encoder.configure(config);
  for (let data of good_data) {
    encoder.encode(data);
    data.close();
  }
  await encoder.flush();

  let txt_config = "sampleRate: " + config.sampleRate
                 + " numberOfChannels: " + config.numberOfChannels;
  assert_equals(error, null, txt_config);
  assert_greater_than(outputs, 0);
  encoder.encode(bad_data);
  await encoder.flush().catch(() => {});
  assert_not_equals(error, null, txt_config);
}

function channelNumberVariationTests() {
  let sample_rate = 48000;
  for (let channels = 1; channels <= 2; channels++) {
    let config = {
      codec: 'opus',
      sampleRate: sample_rate,
      numberOfChannels: channels,
      bitrate: 128000
    };

    let ts = 0;
    let length = sample_rate / 10;
    let data1 = make_audio_data(ts, channels, sample_rate, length);

    ts += Math.floor(data1.duration / 1000000);
    let data2 = make_audio_data(ts, channels, sample_rate, length);
    ts += Math.floor(data2.duration / 1000000);

    let bad_data = make_audio_data(ts, channels + 1, sample_rate, length);
    promise_test(async t =>
      checkEncodingError(config, [data1, data2], bad_data),
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
    let data1 = make_audio_data(ts, channels, sample_rate, length);

    ts += Math.floor(data1.duration / 1000000);
    let data2 = make_audio_data(ts, channels, sample_rate, length);
    ts += Math.floor(data2.duration / 1000000);

    let bad_data = make_audio_data(ts, channels, sample_rate + 333, length);
    promise_test(async t =>
      checkEncodingError(config, [data1, data2], bad_data),
      "Sample rate variation: " + sample_rate);
  }
}
sampleRateVariationTests();

promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 1;
  let data_count = 10;
  let input_data = [];
  let output_data = [];

  let decoder_init = {
    error: t.unreached_func("Decode error"),
    output: data => {
      output_data.push(data);
    }
  };
  let decoder = new AudioDecoder(decoder_init);

  let encoder_init = {
    error: t.unreached_func("Encoder error"),
    output: (chunk, metadata) => {
      let config = metadata.decoderConfig;
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
  const data_duration_s = total_duration_s / data_count;
  const data_length = data_duration_s * config.sampleRate;
  for (let i = 0; i < data_count; i++) {
    let data = make_audio_data(timestamp_us, config.numberOfChannels,
      config.sampleRate, data_length);
    input_data.push(data);
    encoder.encode(data);
    timestamp_us += data_duration_s * 1_000_000;
  }
  await encoder.flush();
  encoder.close();
  await decoder.flush();
  decoder.close();


  let total_input = join_audio_data(input_data);
  let frames_per_plane = total_input.length / config.numberOfChannels;

  let total_output = join_audio_data(output_data);

  let base_input = input_data[0];
  let base_output = output_data[0];

  // TODO: Convert formats to simplify conversions, once
  // https://github.com/w3c/webcodecs/issues/232 is resolved.
  assert_equals(base_input.format, "f32-planar");
  assert_equals(base_output.format, "f32");

  assert_equals(base_output.numberOfChannels, config.numberOfChannels);
  assert_equals(base_output.sampleRate, sample_rate);

  // Output can be slightly longer that the input due to padding
  assert_greater_than_equal(total_output.length, total_input.length);

  // Compare waveform before and after encoding
  for (let channel = 0; channel < base_input.numberOfChannels; channel++) {

    let plane_start = channel * frames_per_plane;
    let input_plane = total_input.slice(
        plane_start, plane_start + frames_per_plane);

    for (let i = 0; i < base_input.numberOfFrames; i += 10) {
      // Instead of de-interleaving the data, directly look into |total_output|
      // for the sample we are interested in.
      let ouput_index = i * base_input.numberOfChannels + channel;

      // Checking only every 10th sample to save test time in slow
      // configurations like MSAN etc.
      assert_approx_equals(
          input_plane[i], total_output[ouput_index], 0.5,
          'Difference between input and output is too large.' +
              ' index: ' + i + ' channel: ' + channel +
              ' input: ' + input_plane[i] +
              ' output: ' + total_output[ouput_index]);
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
    output: (chunk, metadata) => {
      let config = metadata.decoderConfig;
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

  let large_data = make_audio_data(0, encoder_config.numberOfChannels,
    encoder_config.sampleRate, encoder_config.sampleRate);
  encoder.encode(large_data);
  await encoder.flush();

  // Large data produced more than one output, and we've got decoder_config
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
  encoder.encode(large_data);
  await encoder.flush();

  // After reconfiguring encoder should produce decoder config again
  assert_greater_than(output_count, 1);
  assert_not_equals(decoder_config, null);
  assert_not_equals(decoder_config.description, null);
  encoder.close();
}, "Emit decoder config and extra data.");

promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 1;
  let data_count = 100;
  let init = getDefaultCodecInit(t);
  init.output = (chunk, metadata) => {}

  let encoder = new AudioEncoder(init);

  // No encodes yet.
  assert_equals(encoder.encodeQueueSize, 0);

  let config = {
    codec: 'opus',
    sampleRate: sample_rate,
    numberOfChannels: 2,
    bitrate: 256000 //256kbit
  };
  encoder.configure(config);

  // Still no encodes.
  assert_equals(encoder.encodeQueueSize, 0);

  let datas = [];
  let timestamp_us = 0;
  let data_duration_s = total_duration_s / data_count;
  let data_length = data_duration_s * config.sampleRate;
  for (let i = 0; i < data_count; i++) {
    let data = make_audio_data(timestamp_us, config.numberOfChannels,
      config.sampleRate, data_length);
    datas.push(data);
    timestamp_us += data_duration_s * 1_000_000;
  }

  let lastDequeueSize = Infinity;
  encoder.ondequeue = () => {
    assert_greater_than(lastDequeueSize, 0, "Dequeue event after queue empty");
    assert_greater_than(lastDequeueSize, encoder.encodeQueueSize,
                        "Dequeue event without decreased queue size");
    lastDequeueSize = encoder.encodeQueueSize;
  };

  for (let data of datas)
    encoder.encode(data);

  assert_greater_than_equal(encoder.encodeQueueSize, 0);
  assert_less_than_equal(encoder.encodeQueueSize, data_count);

  await encoder.flush();
  // We can guarantee that all encodes are processed after a flush.
  assert_equals(encoder.encodeQueueSize, 0);
  // Last dequeue event should fire when the queue is empty.
  assert_equals(lastDequeueSize, 0);

  // Reset this to Infinity to track the decline of queue size for this next
  // batch of encodes.
  lastDequeueSize = Infinity;

  for (let data of datas) {
    encoder.encode(data);
    data.close();
  }

  assert_greater_than_equal(encoder.encodeQueueSize, 0);
  encoder.reset();
  assert_equals(encoder.encodeQueueSize, 0);
}, 'encodeQueueSize test');
