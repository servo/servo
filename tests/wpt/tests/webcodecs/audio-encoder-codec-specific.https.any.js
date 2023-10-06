// META: global=window
// META: script=/webcodecs/utils.js

function make_silent_audio_data(timestamp, channels, sampleRate, frames) {
  let data = new Float32Array(frames*channels);

  return new AudioData({
    timestamp: timestamp,
    data: data,
    numberOfChannels: channels,
    numberOfFrames: frames,
    sampleRate: sampleRate,
    format: "f32-planar",
  });
}

// The Opus DTX flag (discontinuous transmission) reduces the encoding bitrate
// for silence. This test ensures the DTX flag is working properly by encoding
// almost 10s of silence and comparing the bitrate with and without the flag.
promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 10;
  let data_count = 100;
  let normal_outputs = [];
  let dtx_outputs = [];

  let normal_encoder = new AudioEncoder({
    error: e => {
      assert_unreached('error: ' + e);
    },
    output: chunk => {
      normal_outputs.push(chunk);
    }
  });

  let dtx_encoder = new AudioEncoder({
    error: e => {
      assert_unreached('error: ' + e);
    },
    output: chunk => {
      dtx_outputs.push(chunk);
    }
  });

  let config = {
    codec: 'opus',
    sampleRate: sample_rate,
    numberOfChannels: 2,
    bitrate: 256000,  // 256kbit
  };

  let normal_config = {...config, opus: {usedtx: false}};
  let dtx_config = {...config, opus: {usedtx: true}};

  let normal_config_support = await AudioEncoder.isConfigSupported(normal_config);
  assert_implements_optional(normal_config_support.supported, "Opus not supported");

  let dtx_config_support = await AudioEncoder.isConfigSupported(dtx_config);
  assert_implements_optional(dtx_config_support.supported, "Opus DTX not supported");

  // Configure one encoder with and one without the DTX flag
  normal_encoder.configure(normal_config);
  dtx_encoder.configure(dtx_config);

  let timestamp_us = 0;
  let data_duration_s = total_duration_s / data_count;
  let data_length = data_duration_s * config.sampleRate;
  for (let i = 0; i < data_count; i++) {
    let data;

    if (i == 0 || i == (data_count - 1)) {
      // Send real data for the first and last 100ms.
      data = make_audio_data(
          timestamp_us, config.numberOfChannels, config.sampleRate,
          data_length);

    } else {
      // Send silence for the rest of the 10s.
      data = make_silent_audio_data(
          timestamp_us, config.numberOfChannels, config.sampleRate,
          data_length);
    }

    normal_encoder.encode(data);
    dtx_encoder.encode(data);
    data.close();

    timestamp_us += data_duration_s * 1_000_000;
  }

  await Promise.all([normal_encoder.flush(), dtx_encoder.flush()])

  normal_encoder.close();
  dtx_encoder.close();

  // We expect a significant reduction in the number of packets, over ~10s of silence.
  assert_less_than(dtx_outputs.length, (normal_outputs.length / 2));
}, 'Test the Opus DTX flag works.');


// The Opus bitrateMode enum chooses whether we use a constant or variable bitrate.
// This test ensures that VBR/CBR is respected properly by encoding almost 10s of
// silence and comparing the size of the encoded variable or constant bitrates.
promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 10;
  let data_count = 100;
  let vbr_outputs = [];
  let cbr_outputs = [];

  let cbr_encoder = new AudioEncoder({
    error: e => {
      assert_unreached('error: ' + e);
    },
    output: chunk => {
      cbr_outputs.push(chunk);
    }
  });

  let vbr_encoder = new AudioEncoder({
    error: e => {
      assert_unreached('error: ' + e);
    },
    output: chunk => {
      vbr_outputs.push(chunk);
    }
  });

  let config = {
    codec: 'opus',
    sampleRate: sample_rate,
    numberOfChannels: 2,
    bitrate: 256000,  // 256kbit
  };

  let cbr_config = { ...config, bitrateMode: "constant" };
  let vbr_config = { ...config, bitrateMode: "variable" };

  let cbr_config_support = await AudioEncoder.isConfigSupported(cbr_config);
  assert_implements_optional(cbr_config_support.supported, "Opus CBR not supported");

  let vbr_config_support = await AudioEncoder.isConfigSupported(vbr_config);
  assert_implements_optional(vbr_config_support.supported, "Opus VBR not supported");

  // Configure one encoder with VBR and one CBR.
  cbr_encoder.configure(cbr_config);
  vbr_encoder.configure(vbr_config);

  let timestamp_us = 0;
  let data_duration_s = total_duration_s / data_count;
  let data_length = data_duration_s * config.sampleRate;
  for (let i = 0; i < data_count; i++) {
    let data;

    if (i == 0 || i == (data_count - 1)) {
      // Send real data for the first and last 100ms.
      data = make_audio_data(
        timestamp_us, config.numberOfChannels, config.sampleRate,
        data_length);

    } else {
      // Send silence for the rest of the 10s.
      data = make_silent_audio_data(
        timestamp_us, config.numberOfChannels, config.sampleRate,
        data_length);
    }

    vbr_encoder.encode(data);
    cbr_encoder.encode(data);
    data.close();

    timestamp_us += data_duration_s * 1_000_000;
  }

  await Promise.all([cbr_encoder.flush(), vbr_encoder.flush()])

  cbr_encoder.close();
  vbr_encoder.close();

  let vbr_total_bytes = 0;
  vbr_outputs.forEach(chunk => vbr_total_bytes += chunk.byteLength)

  let cbr_total_bytes = 0;
  cbr_outputs.forEach(chunk => cbr_total_bytes += chunk.byteLength)

  // We expect a significant reduction in the size of the packets, over ~10s of silence.
  assert_less_than(vbr_total_bytes, (cbr_total_bytes / 2));
}, 'Test the Opus bitrateMode flag works.');


// The AAC bitrateMode enum chooses whether we use a constant or variable bitrate.
// This test exercises the VBR/CBR paths. Some platforms don't support VBR for AAC,
// and still emit a constant bitrate.
promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 10;
  let data_count = 100;
  let vbr_outputs = [];
  let cbr_outputs = [];

  let cbr_encoder = new AudioEncoder({
    error: e => {
      assert_unreached('error: ' + e);
    },
    output: chunk => {
      cbr_outputs.push(chunk);
    }
  });

  let vbr_encoder = new AudioEncoder({
    error: e => {
      assert_unreached('error: ' + e);
    },
    output: chunk => {
      vbr_outputs.push(chunk);
    }
  });

  let config = {
    codec: 'mp4a.40.2',
    sampleRate: sample_rate,
    numberOfChannels: 2,
    bitrate: 192000,  // 256kbit
  };

  let cbr_config = { ...config, bitrateMode: "constant" };
  let vbr_config = { ...config, bitrateMode: "variable" };

  let cbr_config_support = await AudioEncoder.isConfigSupported(cbr_config);
  assert_implements_optional(cbr_config_support.supported, "AAC CBR not supported");

  let vbr_config_support = await AudioEncoder.isConfigSupported(vbr_config);
  assert_implements_optional(vbr_config_support.supported, "AAC VBR not supported");

  // Configure one encoder with VBR and one CBR.
  cbr_encoder.configure(cbr_config);
  vbr_encoder.configure(vbr_config);

  let timestamp_us = 0;
  let data_duration_s = total_duration_s / data_count;
  let data_length = data_duration_s * config.sampleRate;
  for (let i = 0; i < data_count; i++) {
    let data;

    if (i == 0 || i == (data_count - 1)) {
      // Send real data for the first and last 100ms.
      data = make_audio_data(
        timestamp_us, config.numberOfChannels, config.sampleRate,
        data_length);

    } else {
      // Send silence for the rest of the 10s.
      data = make_silent_audio_data(
        timestamp_us, config.numberOfChannels, config.sampleRate,
        data_length);
    }

    vbr_encoder.encode(data);
    cbr_encoder.encode(data);
    data.close();

    timestamp_us += data_duration_s * 1_000_000;
  }

  await Promise.all([cbr_encoder.flush(), vbr_encoder.flush()])

  cbr_encoder.close();
  vbr_encoder.close();

  let vbr_total_bytes = 0;
  vbr_outputs.forEach(chunk => vbr_total_bytes += chunk.byteLength)

  let cbr_total_bytes = 0;
  cbr_outputs.forEach(chunk => cbr_total_bytes += chunk.byteLength)

  // We'd like to confirm that the encoded size using VBR is less than CBR, but
  // platforms without VBR support will silently revert to CBR (which is
  // technically a subset of VBR).
  assert_less_than_equal(vbr_total_bytes, cbr_total_bytes);
}, 'Test the AAC bitrateMode flag works.');
