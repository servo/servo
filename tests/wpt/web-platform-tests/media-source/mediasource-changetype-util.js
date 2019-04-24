// Copyright © 2018 Chromium authors and World Wide Web Consortium, (Massachusetts Institute of Technology, ERCIM, Keio University, Beihang).

function findSupportedChangeTypeTestTypes(cb) {
  let CHANGE_TYPE_MEDIA_LIST = [
    {
      type: 'video/webm; codecs="vp8"',
      is_video: true,
      url: 'webm/test-v-128k-320x240-24fps-8kfr.webm',
      start_time: 0.0
      // keyframe_interval: N/A since DTS==PTS so overlap-removal of
      // non-keyframe should not produce a buffered range gap.
    },
    {
      type: 'video/webm; codecs="vp9"',
      is_video: true,
      url: 'webm/test-vp9.webm',
      start_time: 0.0
      // keyframe_interval: N/A since DTS==PTS so overlap-removal of
      // non-keyframe should not produce a buffered range gap.
    },
    {
      type: 'video/mp4; codecs="avc1.4D4001"',
      is_video: true,
      url: 'mp4/test-v-128k-320x240-24fps-8kfr.mp4',
      start_time: 0.083333,
      keyframe_interval: 0.333333
    },
    {
      type: 'audio/webm; codecs="vorbis"',
      is_video: false,
      url: 'webm/test-a-128k-44100Hz-1ch.webm',
      start_time: 0.0
      // keyframe_interval: N/A since DTS==PTS so overlap-removal of
      // non-keyframe should not produce a buffered range gap. Also, all frames
      // in this media are key-frames (it is audio).
    },
    {
      type: 'audio/mp4; codecs="mp4a.40.2"',
      is_video: false,
      url: 'mp4/test-a-128k-44100Hz-1ch.mp4',
      start_time: 0.0
      // keyframe_interval: N/A since DTS==PTS so overlap-removal of
      // non-keyframe should not produce a buffered range gap. Also, all frames
      // in this media are key-frames (it is audio).
    },
    {
      type: 'audio/mpeg',
      is_video: false,
      url: 'mp3/sound_5.mp3',
      start_time: 0.0
      // keyframe_interval: N/A since DTS==PTS so overlap-removal of
      // non-keyframe should not produce a buffered range gap. Also, all frames
      // in this media are key-frames (it is audio).
    }
  ];

  let audio_result = [];
  let video_result = [];

  for (let i = 0; i < CHANGE_TYPE_MEDIA_LIST.length; ++i) {
    let media = CHANGE_TYPE_MEDIA_LIST[i];
    if (window.MediaSource && MediaSource.isTypeSupported(media.type)) {
      if (media.is_video === true) {
        video_result.push(media);
      } else {
        audio_result.push(media);
      }
    }
  }

  cb(audio_result, video_result);
}

function appendBuffer(test, sourceBuffer, data) {
  test.expectEvent(sourceBuffer, "update");
  test.expectEvent(sourceBuffer, "updateend");
  sourceBuffer.appendBuffer(data);
}

function trimBuffered(test, mediaElement, sourceBuffer, minimumPreviousDuration, newDuration) {
  assert_less_than(newDuration, minimumPreviousDuration);
  assert_less_than(minimumPreviousDuration, mediaElement.duration);
  test.expectEvent(sourceBuffer, "update");
  test.expectEvent(sourceBuffer, "updateend");
  sourceBuffer.remove(newDuration, Infinity);
}

function trimDuration(test, mediaElement, mediaSource, newDuration) {
  assert_less_than(newDuration, mediaElement.duration);
  test.expectEvent(mediaElement, "durationchange");
  mediaSource.duration = newDuration;
}

function runChangeTypeTest(test, mediaElement, mediaSource, metadataA, dataA, metadataB, dataB) {
  // Some streams, like the MP4 video stream, contain presentation times for
  // frames out of order versus their decode times. If we overlap-append the
  // latter part of such a stream's GOP presentation interval, a significant
  // portion of decode-dependent non-keyframes with earlier presentation
  // intervals could be removed and a presentation time buffered range gap could
  // be introduced. Therefore, we test overlap appends with the overlaps
  // occurring very near to a keyframe's presentation time to reduce the
  // possibility of such a gap. None of the test media is SAP-Type-2, so we
  // don't take any extra care to avoid gaps that may occur when
  // splice-overlapping such GOP sequences that aren't SAP-Type-1.
  // TODO(wolenetz): https://github.com/w3c/media-source/issues/160 could
  // greatly simplify this problem by allowing us play through these small gaps.

  function findSafeOffset(targetTime, overlappedMediaMetadata, overlappedStartTime, overlappingMediaMetadata) {
    assert_greater_than_equal(targetTime, overlappedStartTime);

    let offset = targetTime;
    if ("start_time" in overlappingMediaMetadata) {
      offset -= overlappingMediaMetadata["start_time"];
    }

    // If the media being overlapped is not out-of-order decode, then we can
    // safely use the supplied times.
    if (!("keyframe_interval" in overlappedMediaMetadata)) {
      return { "offset": offset, "adjustedTime": targetTime };
    }

    // Otherwise, we're overlapping media that needs care to prevent introducing
    // a gap. Adjust offset and adjustedTime to make the overlapping media start
    // at the next overlapped media keyframe at or after targetTime.
    let gopsToRetain = Math.ceil((targetTime - overlappedStartTime) / overlappedMediaMetadata["keyframe_interval"]);
    let adjustedTime = overlappedStartTime + gopsToRetain * overlappedMediaMetadata["keyframe_interval"];

    assert_greater_than_equal(adjustedTime, targetTime);
    offset += adjustedTime - targetTime;
    return { "offset": offset, "adjustedTime": adjustedTime };
  }

  let sourceBuffer = mediaSource.addSourceBuffer(metadataA.type);

  appendBuffer(test, sourceBuffer, dataA);
  let lastStart = metadataA["start_time"];
  if (lastStart == null) {
    lastStart = 0.0;
  }

  // changeType A->B and append the first media of B effectively at 0.5 seconds
  // (or at the first keyframe in A at or after 0.5 seconds if it has
  // keyframe_interval defined).
  test.waitForExpectedEvents(() => {
    let safeOffset = findSafeOffset(0.5, metadataA, lastStart, metadataB);
    lastStart = safeOffset["adjustedTime"];
    sourceBuffer.changeType(metadataB.type);
    sourceBuffer.timestampOffset = safeOffset["offset"];
    appendBuffer(test, sourceBuffer, dataB);
  });

  // changeType B->B and append B starting at 1.0 seconds (or at the first
  // keyframe in B at or after 1.0 seconds if it has keyframe_interval defined).
  test.waitForExpectedEvents(() => {
    assert_less_than(lastStart, 1.0);
    let safeOffset = findSafeOffset(1.0, metadataB, lastStart, metadataB);
    lastStart = safeOffset["adjustedTime"];
    sourceBuffer.changeType(metadataB.type);
    sourceBuffer.timestampOffset = safeOffset["offset"];
    appendBuffer(test, sourceBuffer, dataB);
  });

  // changeType B->A and append A starting at 1.5 seconds (or at the first
  // keyframe in B at or after 1.5 seconds if it has keyframe_interval defined).
  test.waitForExpectedEvents(() => {
    assert_less_than(lastStart, 1.5);
    let safeOffset = findSafeOffset(1.5, metadataB, lastStart, metadataA);
    // Retain the previous lastStart because the next block will append data
    // which begins between that start time and this block's start time.
    sourceBuffer.changeType(metadataA.type);
    sourceBuffer.timestampOffset = safeOffset["offset"];
    appendBuffer(test, sourceBuffer, dataA);
  });

  // changeType A->A and append A starting at 1.3 seconds (or at the first
  // keyframe in B at or after 1.3 seconds if it has keyframe_interval defined).
  test.waitForExpectedEvents(() => {
    assert_less_than(lastStart, 1.3);
    // Our next append will begin by overlapping some of metadataB, then some of
    // metadataA.
    let safeOffset = findSafeOffset(1.3, metadataB, lastStart, metadataA);
    sourceBuffer.changeType(metadataA.type);
    sourceBuffer.timestampOffset = safeOffset["offset"];
    appendBuffer(test, sourceBuffer, dataA);
  });

  // Trim duration to 2 seconds, then play through to end.
  test.waitForExpectedEvents(() => {
    trimBuffered(test, mediaElement, sourceBuffer, 2.1, 2);
  });

  test.waitForExpectedEvents(() => {
    trimDuration(test, mediaElement, mediaSource, 2);
  });

  test.waitForExpectedEvents(() => {
    assert_equals(mediaElement.currentTime, 0);
    test.expectEvent(mediaSource, "sourceended");
    test.expectEvent(mediaElement, "play");
    test.expectEvent(mediaElement, "ended");
    mediaSource.endOfStream();
    mediaElement.play();
  });

  test.waitForExpectedEvents(() => {
    test.done();
  });
}

function mediaSourceChangeTypeTest(metadataA, metadataB, description) {
  mediasource_test((test, mediaElement, mediaSource) => {
    mediaElement.pause();
    mediaElement.addEventListener('error', test.unreached_func("Unexpected event 'error'"));
    MediaSourceUtil.loadBinaryData(test, metadataA.url, (dataA) => {
      MediaSourceUtil.loadBinaryData(test, metadataB.url, (dataB) => {
        runChangeTypeTest(test, mediaElement, mediaSource, metadataA, dataA, metadataB, dataB);
      });
    });
  }, description);
}
