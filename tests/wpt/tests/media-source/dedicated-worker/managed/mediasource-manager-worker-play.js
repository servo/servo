importScripts('../mediasource-message-util.js');
// For rangesToArray(), subtractRanges() and formatRanges().
importScripts('../../managed/mediasource-manager-util.js');

// testharness.js is not used in this worker, as it would communicate with the
// main document's harness over the same postMessage channel.

const MEDIA_LIST = [
  { url: '../../mp4/test.mp4', type: 'video/mp4; codecs="mp4a.40.2,avc1.4d400d"' },
  { url: '../../webm/test.webm', type: 'video/webm; codecs="vp8, vorbis"' },
];

function fail(info) {
  postMessage({ subject: messageSubject.ERROR, info });
}

function check(condition, info) {
  if (!condition)
    fail(info);
  return condition;
}

function rangesToString(timeRanges) {
  return formatRanges(rangesToArray(timeRanges));
}

function nextEvent(target, type) {
  return new Promise(resolve => target.addEventListener(type, resolve, { once: true }));
}

function waitForUpdateEnd(sourceBuffer) {
  return new Promise((resolve, reject) => {
    sourceBuffer.addEventListener('updateend', resolve, { once: true });
    sourceBuffer.addEventListener('error', () => reject('SourceBuffer error'), { once: true });
  });
}

async function run() {
  if (!check(self.ManagedMediaSource, 'ManagedMediaSource is not exposed in DedicatedWorker'))
    return;
  if (!check(self.BufferedChangeEvent, 'BufferedChangeEvent is not exposed in DedicatedWorker'))
    return;
  if (!check(self.ManagedSourceBuffer, 'ManagedSourceBuffer is not exposed in DedicatedWorker'))
    return;

  const media = MEDIA_LIST.find(item => ManagedMediaSource.isTypeSupported(item.type));
  if (!check(media, 'No supported test media'))
    return;
  const mediaData = fetch(media.url).then(response => response.arrayBuffer());

  const source = new ManagedMediaSource();
  check(source.streaming === false, 'streaming is initially false');
  const sourceOpen = nextEvent(source, 'sourceopen');
  const startStreaming = nextEvent(source, 'startstreaming');
  const handle = source.handle;
  postMessage({ subject: messageSubject.HANDLE, info: handle }, { transfer: [handle] });

  await sourceOpen;
  await startStreaming;
  check(source.streaming === true, 'streaming is true after startstreaming');

  const sourceBuffer = source.addSourceBuffer(media.type);
  check(sourceBuffer instanceof ManagedSourceBuffer, 'addSourceBuffer returns a ManagedSourceBuffer');

  // Append the whole media.
  let bufferedChange = nextEvent(sourceBuffer, 'bufferedchange');
  let updateEnd = waitForUpdateEnd(sourceBuffer);
  sourceBuffer.appendBuffer(await mediaData);
  await updateEnd;
  const fullyBuffered = rangesToString(sourceBuffer.buffered);
  let event = await bufferedChange;
  check(event instanceof BufferedChangeEvent, 'bufferedchange is a BufferedChangeEvent');
  check(rangesToString(event.addedRanges) === fullyBuffered,
      `append: addedRanges ${rangesToString(event.addedRanges)} should be ${fullyBuffered}`);
  check(event.removedRanges.length === 0, 'append: removedRanges should be empty');

  const constructed = new BufferedChangeEvent('bufferedchange', { addedRanges: event.removedRanges, removedRanges: sourceBuffer.buffered });
  check(constructed.addedRanges.length === 0 && rangesToString(constructed.removedRanges) === fullyBuffered,
      'BufferedChangeEvent constructed in a worker reports the provided ranges');

  // Shorten the presentation to keep playback short.
  const previous = rangesToArray(sourceBuffer.buffered);
  bufferedChange = nextEvent(sourceBuffer, 'bufferedchange');
  updateEnd = waitForUpdateEnd(sourceBuffer);
  sourceBuffer.remove(0.5, Infinity);
  await updateEnd;
  const expectedRemoved = formatRanges(subtractRanges(previous, rangesToArray(sourceBuffer.buffered)));
  event = await bufferedChange;
  check(event.addedRanges.length === 0, 'remove: addedRanges should be empty');
  check(rangesToString(event.removedRanges) === expectedRemoved,
      `remove: removedRanges ${rangesToString(event.removedRanges)} should be ${expectedRemoved}`);

  source.duration = 0.5;
  source.endOfStream();
  postMessage({ subject: messageSubject.WORKER_DONE });
}

run().catch(error => fail(String(error)));
