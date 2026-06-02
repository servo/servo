// Copyright © 2018 Chromium authors and World Wide Web Consortium, (Massachusetts Institute of Technology, ERCIM, Keio University, Beihang).

function findSupportedChangeTypeTestTypes(cb) {
  // Changetype test media metadata.
  // type: fully specified mime type (and codecs substring if the bytestream
  //   format does not forbid codecs parameter). This is required for use with
  //   isTypeSupported, and if supported, should work with both addSourceBuffer
  //   and changeType (unless implementation has restrictions).
  //
  // relaxed_type: possibly ambiguous mime type/subtype without any codecs
  //   substring. This is the same as type minus any codecs substring.
  //
  // mime_subtype: the subtype of the mime type in type and relaxed_type. Across
  //   types registered in the bytestream format registry
  //   (https://www.w3.org/TR/mse-byte-stream-format-registry/), this is
  //   currently sufficient to describe uniquely which test media share the same
  //   bytestream format for use in implicit changeType testing.
  //
  // is_video: All test media currently is single track. This describes whether
  //   or not the track is video.
  //
  // url: Relative location of the test media file.
  //
  // The next two items enable more reliable test media splicing test logic that
  // prevents buffered range gaps at the splice points.
  // start_time: Some test media begins at a time later than 0.0 seconds. This
  //   is the start time of the media.
  // keyframe_interval: Some test media contains out-of-order PTS versus DTS
  //   coded frames. In those cases, a constant keyframe_interval is needed to
  //   prevent severely truncating out-of-order GOPs at splice points.
  let CHANGE_TYPE_MEDIA_LIST = [
    {
      type: 'video/webm; codecs="vp8"',
      relaxed_type: 'video/webm',
      mime_subtype: 'webm',
      is_video: true,
      url: 'webm/test-v-128k-320x240-24fps-8kfr.webm',
      start_time: 0.0
      // keyframe_interval: N/A since DTS==PTS so overlap-removal of
      // non-keyframe should not produce a buffered range gap.
    },
    {
      type: 'video/webm; codecs="vp9"',
      relaxed_type: 'video/webm',
      mime_subtype: 'webm',
      is_video: true,
      url: 'webm/test-vp9.webm',
      start_time: 0.0
      // keyframe_interval: N/A since DTS==PTS so overlap-removal of
      // non-keyframe should not produce a buffered range gap.
    },
    {
      type: 'video/mp4; codecs="avc1.4D4001"',
      relaxed_type: 'video/mp4',
      mime_subtype: 'mp4',
      is_video: true,
      url: 'mp4/test-v-128k-320x240-24fps-8kfr.mp4',
      start_time: 0.083333,
      keyframe_interval: 0.333333
    },
    {
      type: 'audio/webm; codecs="vorbis"',
      relaxed_type: 'audio/webm',
      mime_subtype: 'webm',
      is_video: false,
      url: 'webm/test-a-128k-44100Hz-1ch.webm',
      start_time: 0.0
      // keyframe_interval: N/A since DTS==PTS so overlap-removal of
      // non-keyframe should not produce a buffered range gap. Also, all frames
      // in this media are key-frames (it is audio).
    },
    {
      type: 'audio/mp4; codecs="mp4a.40.2"',
      relaxed_type: 'audio/mp4',
      mime_subtype: 'mp4',
      is_video: false,
      url: 'mp4/test-a-128k-44100Hz-1ch.mp4',
      start_time: 0.0
      // keyframe_interval: N/A since DTS==PTS so overlap-removal of
      // non-keyframe should not produce a buffered range gap. Also, all frames
      // in this media are key-frames (it is audio).
    },
    {
      type: 'audio/mpeg',
      relaxed_type: 'audio/mpeg',
      mime_subtype: 'mpeg',
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

function trimBuffered(test, mediaSource, sourceBuffer, minimumPreviousDuration, newDuration, skip_duration_prechecks) {
  if (!skip_duration_prechecks) {
    assert_less_than(newDuration, minimumPreviousDuration,
        "trimBuffered newDuration must be less than minimumPreviousDuration");
    assert_less_than(minimumPreviousDuration, mediaSource.duration,
        "trimBuffered minimumPreviousDuration must be less than mediaSource.duration");
  }
  test.expectEvent(sourceBuffer, "update");
  test.expectEvent(sourceBuffer, "updateend");
  sourceBuffer.remove(newDuration, Infinity);
}

function trimDuration(test, mediaElement, mediaSource, newDuration, skip_duration_prechecks) {
  if (!skip_duration_prechecks) {
    assert_less_than(newDuration, mediaSource.duration,
        "trimDuration newDuration must be less than mediaSource.duration");
  }
  test.expectEvent(mediaElement, "durationchange");
  mediaSource.duration = newDuration;
}

function runChangeTypeTest(test, mediaElement, mediaSource, metadataA, typeA, dataA, metadataB, typeB, dataB,
                           implicit_changetype, negative_test) {
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
  //
  // typeA and typeB may be underspecified for use with isTypeSupported, but
  // this helper does not use isTypeSupported. typeA and typeB must work (even
  // if missing codec specific substrings) with addSourceBuffer (just typeA) and
  // changeType (both typeA and typeB).
  //
  // See also mediaSourceChangeTypeTest's options argument for the meanings of
  // implicit_changetype and negative_test.

  function findSafeOffset(targetTime, overlappedMediaMetadata, overlappedStartTime, overlappingMediaMetadata) {
    assert_greater_than_equal(targetTime, overlappedStartTime,
        "findSafeOffset targetTime must be greater than or equal to overlappedStartTime");

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

    assert_greater_than_equal(adjustedTime, targetTime,
        "findSafeOffset adjustedTime must be greater than or equal to targetTime");
    offset += adjustedTime - targetTime;
    return { "offset": offset, "adjustedTime": adjustedTime };
  }

  // Note, none of the current negative changeType tests should fail the initial addSourceBuffer.
  let sourceBuffer = mediaSource.addSourceBuffer(typeA);

  // Add error event listeners to sourceBuffer. The caller of this helper may
  // also have installed error event listeners on mediaElement.
  if (negative_test) {
    sourceBuffer.addEventListener("error", test.step_func_done());
  } else {
    sourceBuffer.addEventListener("error", test.unreached_func("Unexpected event 'error'"));
  }

  // In either negative test or not, the first appendBuffer should succeed.
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
    if (!implicit_changetype) {
      try { sourceBuffer.changeType(typeB); } catch(err) {
        if (negative_test)
          test.done();
        else
          throw err;
      }
    }
    sourceBuffer.timestampOffset = safeOffset["offset"];
    appendBuffer(test, sourceBuffer, dataB);
  });

  // changeType B->B and append B starting at 1.0 seconds (or at the first
  // keyframe in B at or after 1.0 seconds if it has keyframe_interval defined).
  test.waitForExpectedEvents(() => {
    assert_less_than(lastStart, 1.0,
        "changeType B->B lastStart must be less than 1.0");
    let safeOffset = findSafeOffset(1.0, metadataB, lastStart, metadataB);
    lastStart = safeOffset["adjustedTime"];
    if (!implicit_changetype) {
      try { sourceBuffer.changeType(typeB); } catch(err) {
        if (negative_test)
          test.done();
        else
          throw err;
      }
    }
    sourceBuffer.timestampOffset = safeOffset["offset"];
    appendBuffer(test, sourceBuffer, dataB);
  });

  // changeType B->A and append A starting at 1.5 seconds (or at the first
  // keyframe in B at or after 1.5 seconds if it has keyframe_interval defined).
  test.waitForExpectedEvents(() => {
    assert_less_than(lastStart, 1.5,
        "changeType B->A lastStart must be less than 1.5");
    let safeOffset = findSafeOffset(1.5, metadataB, lastStart, metadataA);
    // Retain the previous lastStart because the next block will append data
    // which begins between that start time and this block's start time.
    if (!implicit_changetype) {
      try { sourceBuffer.changeType(typeA); } catch(err) {
        if (negative_test)
          test.done();
        else
          throw err;
      }
    }
    sourceBuffer.timestampOffset = safeOffset["offset"];
    appendBuffer(test, sourceBuffer, dataA);
  });

  // changeType A->A and append A starting at 1.3 seconds (or at the first
  // keyframe in B at or after 1.3 seconds if it has keyframe_interval defined).
  test.waitForExpectedEvents(() => {
    assert_less_than(lastStart, 1.3,
        "changeType A->A lastStart must be less than 1.3");
    // Our next append will begin by overlapping some of metadataB, then some of
    // metadataA.
    let safeOffset = findSafeOffset(1.3, metadataB, lastStart, metadataA);
    if (!implicit_changetype) {
      try { sourceBuffer.changeType(typeA); } catch(err) {
        if (negative_test)
          test.done();
        else
          throw err;
      }
    }
    sourceBuffer.timestampOffset = safeOffset["offset"];
    appendBuffer(test, sourceBuffer, dataA);
  });

  // Trim duration to 2 seconds, then play through to end.
  test.waitForExpectedEvents(() => {
    // If negative testing, then skip fragile assertions.
    trimBuffered(test, mediaSource, sourceBuffer, 2.1, 2, negative_test);
  });

  test.waitForExpectedEvents(() => {
    // If negative testing, then skip fragile assertions.
    trimDuration(test, mediaElement, mediaSource, 2, negative_test);
  });

  test.waitForExpectedEvents(() => {
    assert_equals(mediaElement.currentTime, 0, "currentTime must be 0");
    test.expectEvent(mediaSource, "sourceended");
    test.expectEvent(mediaElement, "play");
    test.expectEvent(mediaElement, "ended");
    mediaSource.endOfStream();
    mediaElement.play();
  });

  test.waitForExpectedEvents(() => {
    if (negative_test)
      assert_unreached("Received 'ended' while negative testing.");
    else
      test.done();
  });
}

// options.use_relaxed_mime_types : boolean (defaults to false).
//   If true, the initial addSourceBuffer and any changeType calls will use the
//   relaxed_type in metadataA and metadataB instead of the full type in the
//   metadata.
// options.implicit_changetype : boolean (defaults to false).
//   If true, no changeType calls will be used. Instead, the test media files
//   are expected to begin with an initialization segment and end at a segment
//   boundary (no abort() call is issued by this test to reset the
//   SourceBuffer's parser).
// options.negative_test : boolean (defaults to false).
//   If true, the test is expected to hit error amongst one of the following
//   areas: addSourceBuffer, appendBuffer (synchronous or asynchronous error),
//   changeType, playback to end of buffered media. If 'ended' is received
//   without error otherwise already occurring, then fail the test. Otherwise,
//   pass the test on receipt of error. Continue to consider timeouts as test
//   failures.
function mediaSourceChangeTypeTest(metadataA, metadataB, description, options = {}) {
  mediasource_test((test, mediaElement, mediaSource) => {
    let typeA = metadataA.type;
    let typeB = metadataB.type;
    if (options.hasOwnProperty("use_relaxed_mime_types") &&
        options.use_relaxed_mime_types === true) {
      typeA = metadataA.relaxed_type;
      typeB = metadataB.relaxed_type;
    }
    let implicit_changetype = options.hasOwnProperty("implicit_changetype") &&
        options.implicit_changetype === true;
    let negative_test = options.hasOwnProperty("negative_test") &&
        options.negative_test === true;

    mediaElement.pause();
    if (negative_test) {
      mediaElement.addEventListener("error", test.step_func_done());
    } else {
      mediaElement.addEventListener("error",
          test.unreached_func("Unexpected event 'error'"));
    }
    MediaSourceUtil.loadBinaryData(test, metadataA.url, (dataA) => {
      MediaSourceUtil.loadBinaryData(test, metadataB.url, (dataB) => {
        runChangeTypeTest(
            test, mediaElement, mediaSource,
            metadataA, typeA, dataA, metadataB, typeB, dataB,
            implicit_changetype, negative_test);
      });
    });
  }, description);
}
