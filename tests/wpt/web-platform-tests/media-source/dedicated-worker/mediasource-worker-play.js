importScripts("mediasource-worker-util.js");

// Note, we do not use testharness.js utilities within the worker context
// because it also communicates using postMessage to the main HTML document's
// harness, and would confuse the test case message parsing there.

onmessage = function(evt) {
  postMessage({ subject: messageSubject.ERROR, info: "No message expected by Worker"});
};

let util = new MediaSourceWorkerUtil();

util.mediaSource.addEventListener("sourceopen", () => {
  URL.revokeObjectURL(util.mediaSourceObjectUrl);
  sourceBuffer = util.mediaSource.addSourceBuffer(util.mediaMetadata.type);
  sourceBuffer.onerror = (err) => {
    postMessage({ subject: messageSubject.ERROR, info: err });
  };
  sourceBuffer.onupdateend = () => {
    // Reset the parser. Unnecessary for this buffering, except helps with test
    // coverage.
    sourceBuffer.abort();
    // Shorten the buffered media and test playback duration to avoid timeouts.
    sourceBuffer.remove(0.5, Infinity);
    sourceBuffer.onupdateend = () => {
      util.mediaSource.duration = 0.5;
      // Issue changeType to the same type that we've already buffered.
      // Unnecessary for this buffering, except helps with test coverage.
      sourceBuffer.changeType(util.mediaMetadata.type);
      util.mediaSource.endOfStream();
      // Sanity check the duration.
      // Unnecessary for this buffering, except helps with test coverage.
      var duration = util.mediaSource.duration;
      if (isNaN(duration) || duration <= 0.0 || duration >= 1.0) {
        postMessage({
          subject: messageSubject.ERROR,
          info: "mediaSource.duration " + duration + " is not within expected range (0,1)"
        });
      }
    };
  };
  util.mediaLoadPromise.then(mediaData => { sourceBuffer.appendBuffer(mediaData); },
                             err => { postMessage({ subject: messageSubject.ERROR, info: err }) });
}, { once : true });

postMessage({ subject: messageSubject.OBJECT_URL, info: util.mediaSourceObjectUrl });
