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

promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 2;
  let frame_count = 20;
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
  for (let i = 0; i < frame_count; i++) {
    let frame_duration_s = total_duration_s / frame_count;
    let length = frame_duration_s * config.sampleRate;
    let frame = make_audio_frame(timestamp_us, config.numberOfChannels,
                                 config.sampleRate, length);
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
