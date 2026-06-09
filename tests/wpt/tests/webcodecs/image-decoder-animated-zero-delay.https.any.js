// META: global=window,dedicatedworker

// Bug: An animated GIF whose first frame has delay_time=0 is incorrectly
// detected as non-animated by the metadata decoder. This causes ImageDecoder
// to report frameCount=1 and reject decode requests for subsequent frames.

promise_test(async t => {
  let support = await ImageDecoder.isTypeSupported('image/gif');
  assert_implements_optional(support, 'Optional codec image/gif not supported.');

  let response = await fetch('animated-zero-delay.gif');
  let buffer = await response.arrayBuffer();
  let decoder = new ImageDecoder({data: buffer, type: 'image/gif'});

  await decoder.tracks.ready;
  assert_equals(decoder.tracks.length, 1, 'Should have one track');

  let track = decoder.tracks.selectedTrack;
  assert_true(track.animated, 'Track should be detected as animated');
  assert_equals(track.frameCount, 2, 'Should report 2 frames');

  let result0 = await decoder.decode({frameIndex: 0});
  assert_true(result0.complete, 'Frame 0 should be complete');
  assert_equals(result0.image.codedWidth, 2);
  assert_equals(result0.image.codedHeight, 2);

  let result1 = await decoder.decode({frameIndex: 1});
  assert_true(result1.complete, 'Frame 1 should be complete');
  assert_equals(result1.image.codedWidth, 2);
  assert_equals(result1.image.codedHeight, 2);
}, 'Test animated GIF with zero first-frame delay decodes all frames');

promise_test(async t => {
  let support = await ImageDecoder.isTypeSupported('image/gif');
  assert_implements_optional(support, 'Optional codec image/gif not supported.');

  let response = await fetch('animated-zero-delay.gif');
  let buffer = await response.arrayBuffer();
  let decoder = new ImageDecoder({data: buffer, type: 'image/gif'});

  await decoder.completed;

  let track = decoder.tracks.selectedTrack;
  assert_true(track.animated, 'Track should be animated after completed');
  assert_equals(track.frameCount, 2, 'Should report 2 frames after completed');
  assert_equals(track.repetitionCount, Infinity,
      'Should report infinite repetitions (loop=0)');
}, 'Test animated GIF with zero first-frame delay reports correct metadata');
