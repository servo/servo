// This is similar to mediasource-worker-play.js, except that the buffering is
// longer and done in tiny chunks to enable a better chance of the main thread
// detaching the element while interesting buffering work is still occurring. To
// assist the main thread understanding when the buffering has started already
// or has completed already, we also perform extra messaging.
importScripts("mediasource-worker-util.js");

onmessage = function(evt) {
  postMessage({ subject: messageSubject.ERROR, info: "No message expected by Worker" });
};

let util = new MediaSourceWorkerUtil();

let sentStartedBufferingMessage = false;

util.mediaSource.addEventListener("sourceopen", () => {
  URL.revokeObjectURL(util.mediaSourceObjectUrl);
  let sourceBuffer;
  try {
    sourceBuffer = util.mediaSource.addSourceBuffer(util.mediaMetadata.type);
  }  catch(e) {
    // Detachment may have already begun, so allow exception here.
    // TODO(https://crbug.com/878133): Consider a distinct readyState for the case
    // where exception occurs due to "Worker MediaSource attachment is closing".
    // That would assist API users and narrow the exception handling here.
    return;
  }

  sourceBuffer.onerror = (err) => {
    postMessage({ subject: messageSubject.ERROR, info: err });
  };
  util.mediaLoadPromise.then(mediaData => bufferInto(sourceBuffer, mediaData, 100, 0),
                             err => { postMessage({ subject: messageSubject.ERROR, info: err }) } );
}, { once : true });

postMessage({ subject: messageSubject.OBJECT_URL, info: util.mediaSourceObjectUrl} );

// Append increasingly large pieces at a time, starting/continuing at |position|.
// This allows buffering the test media without timeout, but also with enough
// operations to gain coverage on detachment concurrency with append.
function bufferInto(sourceBuffer, mediaData, appendSize, position) {
  if (position >= mediaData.byteLength) {
    postMessage({ subject: messageSubject.FINISHED_BUFFERING });
    try {
      util.mediaSource.endOfStream();
    }  catch(e) {
      // Detachment may have already begun, so allow exception here.
      // TODO(https://crbug.com/878133): Consider a distinct readyState for the case
      // where exception occurs due to "Worker MediaSource attachment is closing".
      // That would assist API users and narrow the exception handling here.
      // FALL-THROUGH - return.
    }
    return;
  }

  var nextPosition = position + appendSize;
  const pieceToAppend = mediaData.slice(position, nextPosition);
  position = nextPosition;
  appendSize += 100;

  sourceBuffer.addEventListener("updateend", () => {
    if (!sentStartedBufferingMessage) {
      postMessage({ subject: messageSubject.STARTED_BUFFERING});
      sentStartedBufferingMessage = true;
    }
    bufferInto(sourceBuffer, mediaData, appendSize, position);
  }, { once : true });

  try {
    sourceBuffer.appendBuffer(pieceToAppend);
  }  catch(e) {
    // Detachment may have already begun, so allow exception here.
    // TODO(https://crbug.com/878133): Consider a distinct readyState for the case
    // where exception occurs due to "Worker MediaSource attachment is closing".
    // That would assist API users and narrow the exception handling here.
    // FALL-THROUGH - return.
  }
}
