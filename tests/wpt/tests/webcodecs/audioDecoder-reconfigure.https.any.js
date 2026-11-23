// META: global=window,dedicatedworker
// META: script=audioDecoder-codec-specific-setup.js
// META: variant=?adts_aac
// META: variant=?mp4_aac
// META: variant=?mp3
// META: variant=?opus
// META: variant=?pcm_alaw
// META: variant=?pcm_ulaw
// META: variant=?pcm_u8
// META: variant=?pcm_s16
// META: variant=?pcm_s24
// META: variant=?pcm_s32
// META: variant=?pcm_f32
// META: variant=?flac
// META: variant=?vorbis

// CHUNKS again, with timestamps following those of CHUNKS.
function secondSegment() {
  const last = CHUNKS[CHUNKS.length - 1];
  const offset = last.timestamp + last.duration;
  return CHUNKS.map((chunk, i) => new EncodedAudioChunk({
    type: 'key',
    timestamp: chunk.timestamp + offset,
    duration: chunk.duration,
    data: CHUNK_DATA[i],
  }));
}

// Output timestamps of a new decoder given `chunks`.
async function referenceTimestamps(t, chunks) {
  const timestamps = [];
  const decoder = createAudioDecoder(t, {
    output: data => {
      timestamps.push(data.timestamp);
      data.close();
    },
  });
  decoder.configure(CONFIG);
  for (const chunk of chunks)
    decoder.decode(chunk);
  await decoder.flush();
  decoder.close();
  return timestamps;
}

promise_test(async t => {
  const second = secondSegment();
  const expected = [...await referenceTimestamps(t, CHUNKS), ...await referenceTimestamps(t, second)];

  const timestamps = [];
  const decoder = createAudioDecoder(t, {
    output: data => {
      timestamps.push(data.timestamp);
      data.close();
    },
  });
  decoder.configure(CONFIG);
  for (const chunk of CHUNKS)
    decoder.decode(chunk);
  decoder.configure(CONFIG);
  for (const chunk of second)
    decoder.decode(chunk);
  await decoder.flush();
  decoder.close();

  assert_array_equals(timestamps, expected, 'output timestamps');
}, 'Test configure() between decode() calls outputs the audio decoded with the previous configuration');

promise_test(async t => {
  const second = secondSegment();
  const expected = await referenceTimestamps(t, second);

  let resetCalled = false;
  const timestampsAfterReset = [];
  let resolveFlushed;
  const flushed = new Promise(resolve => resolveFlushed = resolve);
  const decoder = createAudioDecoder(t, {
    output: data => {
      const timestamp = data.timestamp;
      data.close();
      if (resetCalled) {
        timestampsAfterReset.push(timestamp);
        return;
      }
      // The remaining audio of the first chunks is still being output.
      decoder.configure(CONFIG);
      decoder.reset();
      resetCalled = true;
      decoder.configure(CONFIG);
      for (const chunk of second)
        decoder.decode(chunk);
      resolveFlushed(decoder.flush());
    },
  });

  decoder.configure(CONFIG);
  for (const chunk of CHUNKS)
    decoder.decode(chunk);
  await flushed;
  decoder.close();

  assert_array_equals(timestampsAfterReset, expected, 'output timestamps after reset()');
}, 'Test reset() while configure() waits for the previous decoder discards its audio');
