// Helpers for ManagedMediaSource tests.
// This file does not depend on MediaSource being exposed, since some user
// agents only expose ManagedMediaSource.

const MANAGED_MEDIA_LIST = [
  {
    // Muxed audio and video, each segment ending at a different time per track.
    // Segment offsets are those of mediasource-util.js's SEGMENT_INFO_LIST.
    muxed: {
      url: '../mp4/test.mp4',
      type: 'video/mp4; codecs="mp4a.40.2,avc1.4d400d"',
      init: { offset: 0, size: 1413 },
      media: [
        { offset: 1413, size: 24034 },
        { offset: 25447, size: 21757 },
      ],
    },
    // Muxed audio and video whose audio track ends before its video track.
    unequalTrackEnds: {
      url: '../mp4/test-av-384k-44100Hz-1ch-320x240-30fps-10kfr.mp4',
      type: 'video/mp4;codecs="avc1.4D4001,mp4a.40.2"',
    },
    audioOnly: {
      url: '../mp4/test-a-128k-44100Hz-1ch.mp4',
      type: 'audio/mp4;codecs="mp4a.40.2"',
    },
  },
  {
    muxed: {
      url: '../webm/test.webm',
      type: 'video/webm; codecs="vp8, vorbis"',
      init: { offset: 0, size: 4116 },
      media: [
        { offset: 4116, size: 26583 },
        { offset: 30699, size: 20555 },
      ],
    },
    unequalTrackEnds: {
      url: '../webm/test-av-384k-44100Hz-1ch-320x240-30fps-10kfr.webm',
      type: 'video/webm;codecs="vp8,vorbis"',
    },
    audioOnly: {
      url: '../webm/test-a-128k-44100Hz-1ch.webm',
      type: 'audio/webm;codecs="vorbis"',
    },
  },
];

function getManagedMedia() {
  if (!self.ManagedMediaSource)
    return null;
  for (const media of MANAGED_MEDIA_LIST) {
    if (Object.values(media).every(item => ManagedMediaSource.isTypeSupported(item.type)))
      return media;
  }
  return null;
}

const MANAGED_MEDIA = getManagedMedia();

async function fetchMediaData(url) {
  const response = await fetch(url);
  assert_true(response.ok, `fetching ${url} succeeded`);
  return new Uint8Array(await response.arrayBuffer());
}

function extractSegment(data, info) {
  return data.subarray(info.offset, info.offset + info.size);
}

// Creates a ManagedMediaSource and attaches it to a new video element.
// Listeners that must observe events fired as part of attaching (such as
// startstreaming) can be registered on the returned source before awaiting
// sourceOpen, as those events are dispatched asynchronously.
function attachManagedMediaSource(t) {
  assert_implements(self.ManagedMediaSource, 'ManagedMediaSource is supported');
  assert_not_equals(MANAGED_MEDIA, null, 'Test media is supported');

  const video = document.createElement('video');
  // Some user agents require remote playback to be disabled, or an alternative
  // source to be provided, before a ManagedMediaSource can open.
  video.disableRemotePlayback = true;
  video.addEventListener('error', t.unreached_func('Unexpected media element error'));
  document.body.appendChild(video);

  const source = new ManagedMediaSource();
  const url = URL.createObjectURL(source);
  const sourceOpen = new Promise(resolve => {
    source.addEventListener('sourceopen', () => {
      URL.revokeObjectURL(url);
      resolve();
    }, { once: true });
  });
  video.src = url;

  t.add_cleanup(() => {
    video.removeAttribute('src');
    video.load();
    video.remove();
  });

  return { video, source, sourceOpen };
}

function waitForEvent(target, type) {
  return new Promise(resolve => target.addEventListener(type, resolve, { once: true }));
}

function appendBufferAsync(sourceBuffer, data) {
  return new Promise((resolve, reject) => {
    sourceBuffer.addEventListener('updateend', resolve, { once: true });
    sourceBuffer.addEventListener('error', () => reject(new Error('append error')), { once: true });
    sourceBuffer.appendBuffer(data);
  });
}

function removeAsync(sourceBuffer, start, end) {
  return new Promise(resolve => {
    sourceBuffer.addEventListener('updateend', resolve, { once: true });
    sourceBuffer.remove(start, end);
  });
}

// Gives tasks that are already queued a chance to run.
function flushTasks(t) {
  return new Promise(resolve => t.step_timeout(() => t.step_timeout(resolve, 0), 0));
}

function rangesToArray(timeRanges) {
  const ranges = [];
  for (let i = 0; i < timeRanges.length; ++i)
    ranges.push([timeRanges.start(i), timeRanges.end(i)]);
  return ranges;
}

// Returns the normalized ranges of |a| with |b| subtracted, where both are
// arrays of [start, end] pairs as returned by rangesToArray().
function subtractRanges(a, b) {
  let result = a.map(range => range.slice());
  for (const [removeStart, removeEnd] of b) {
    const next = [];
    for (const [start, end] of result) {
      if (removeEnd <= start || removeStart >= end) {
        next.push([start, end]);
        continue;
      }
      if (start < removeStart)
        next.push([start, removeStart]);
      if (removeEnd < end)
        next.push([removeEnd, end]);
    }
    result = next;
  }
  return result;
}

function formatRanges(ranges) {
  return '{ ' + ranges.map(([start, end]) => `[${start.toFixed(3)}, ${end.toFixed(3)})`).join(' ') + ' }';
}

function assertTimeRangesEqual(actual, expected, description) {
  assert_equals(formatRanges(rangesToArray(actual)), formatRanges(expected), description);
}

// Records every bufferedchange event fired at |sourceBuffer|.
class BufferedChangeRecorder {
  constructor(sourceBuffer) {
    this.events = [];
    this.waiters = [];
    sourceBuffer.addEventListener('bufferedchange', event => {
      this.events.push(event);
      const waiters = this.waiters;
      this.waiters = [];
      waiters.forEach(resolve => resolve());
    });
  }

  // Resolves once at least |count| events have been recorded.
  async waitForCount(count) {
    while (this.events.length < count)
      await new Promise(resolve => this.waiters.push(resolve));
  }

  // Returns a promise resolving to the first event recorded after this call.
  nextEvent() {
    const index = this.events.length;
    return this.waitForCount(index + 1).then(() => this.events[index]);
  }
}

// Checks that |event| reports the difference between |previous| and |current|,
// both arrays of [start, end] pairs.
function assertBufferedChangeEvent(event, previous, current, description) {
  assert_true(event instanceof BufferedChangeEvent, `${description}: event is a BufferedChangeEvent`);
  assertTimeRangesEqual(event.addedRanges, subtractRanges(current, previous), `${description}: addedRanges`);
  assertTimeRangesEqual(event.removedRanges, subtractRanges(previous, current), `${description}: removedRanges`);
}
