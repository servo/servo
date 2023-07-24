importScripts("mediasource-worker-util.js");

// Note, we do not use testharness.js utilities within the worker context
// because it also communicates using postMessage to the main HTML document's
// harness, and would confuse the test case message parsing there.

let util = new MediaSourceWorkerUtil();
let sourceBuffer;

// Phases of this test case, in sequence:
const testPhase = {
  // Main thread verifies initial unattached HTMLMediaElement duration is NaN
  // and readyState is HAVE_NOTHING, then starts this worker.
  // This worker creates a MediaSource, verifies its initial duration
  // is NaN, creates an object URL for the MediaSource and sends the URL to the
  // main thread.
  kInitial: "Initial",

  // Main thread receives MediaSourceHandle, re-verifies that the media element
  // duration is still NaN and readyState is still HAVE_NOTHING, and then sets
  // the handle as the srcObject of the media element, eventually causing worker
  // mediaSource 'onsourceopen' event dispatch.
  kAttaching: "Awaiting sourceopen event that signals attachment is setup",

  kRequestNaNDurationCheck:
      "Sending request to main thread to verify expected duration of the freshly setup attachment",
  kConfirmNaNDurationResult:
      "Checking that main thread correctly ACK'ed the freshly setup attachment's duration verification request",

  kRequestHaveNothingReadyStateCheck:
      "Sending request to main thread to verify expected readyState of HAVE_NOTHING of the freshly setup attachment",
  kConfirmHaveNothingReadyStateResult:
      "Checking that main thread correctly ACK'ed the freshly setup attachment's readyState HAVE_NOTHING verification request",

  kRequestSetDurationCheck:
      "Sending request to main thread to verify explicitly set duration before any media data has been appended",
  kConfirmSetDurationResult:
      "Checking that main thread correctly ACK'ed the duration verification request of explicitly set duration before any media data has been appended",

  kRequestHaveNothingReadyStateRecheck:
      "Sending request to main thread to recheck that the readyState is still HAVE_NOTHING",
  kConfirmHaveNothingReadyStateRecheckResult:
      "Checking that main thread correctly ACK'ed the request to recheck readyState of HAVE_NOTHING",

  kRequestAwaitNewDurationCheck:
      "Buffering media and then sending request to main thread to await duration reaching the expected value due to buffering",
  kConfirmAwaitNewDurationResult:
      "Checking that main thread correctly ACK'ed the request to await duration reaching the expected value due to buffering",

  kRequestAtLeastHaveMetadataReadyStateCheck:
      "Sending request to main thread to verify expected readyState of at least HAVE_METADATA due to buffering",
  kConfirmAtLeastHaveMetadataReadyStateResult:
      "Checking that main thread correctly ACK'ed the request to verify expected readyState of at least HAVE_METADATA due to buffering",

};

let phase = testPhase.kInitial;

// Setup handler for receipt of attachment completion.
util.mediaSource.addEventListener("sourceopen", () => {
  assert(phase === testPhase.kAttaching, "Unexpected sourceopen received by Worker mediaSource.");
  phase = testPhase.kRequestNaNDurationCheck;
  processPhase();
}, { once : true });

// Setup handler for receipt of acknowledgement of successful verifications from
// main thread. |ackVerificationData| contains the round-tripped verification
// request that the main thread just sent, and is used in processPhase to ensure
// the ACK for this phase matched the request for verification.
let ackVerificationData;
onmessage = e => {
  if (e.data === undefined || e.data.subject !== messageSubject.ACK_VERIFIED || e.data.info === undefined) {
    postMessage({
      subject: messageSubject.ERROR,
      info: "Invalid message received by Worker"
    });
    return;
  }

  ackVerificationData = e.data.info;
  processPhase(/* isResponseToAck */ true);
};

processPhase();


// Returns true if checks succeed, false otherwise.
function checkAckVerificationData(expectedRequest) {

  // Compares only subject and info fields. Uses logic similar to testharness.js's
  // same_value(x,y) to correctly handle NaN, but doesn't distinguish +0 from -0.
  function messageValuesEqual(m1, m2) {
    if (m1.subject !== m1.subject) {
      // NaN case
      if (m2.subject === m2.subject)
        return false;
    } else if (m1.subject !== m2.subject) {
      return false;
    }

    if (m1.info !== m1.info) {
      // NaN case
      return (m2.info !== m2.info);
    }

    return m1.info === m2.info;
  }

  if (messageValuesEqual(expectedRequest, ackVerificationData)) {
    ackVerificationData = undefined;
    return true;
  }

  postMessage({
    subject: messageSubject.ERROR,
    info: "ACK_VERIFIED message from main thread was for a mismatching request for this phase. phase=[" + phase +
          "], expected request that would produce ACK in this phase=[" + JSON.stringify(expectedRequest) +
          "], actual request reported with the ACK=[" + JSON.stringify(ackVerificationData) + "]"
  });

  ackVerificationData = undefined;
  return false;
}

function bufferMediaAndSendDurationVerificationRequest() {
  sourceBuffer = util.mediaSource.addSourceBuffer(util.mediaMetadata.type);
  sourceBuffer.onerror = (err) => {
    postMessage({ subject: messageSubject.ERROR, info: err });
  };
  sourceBuffer.onupdateend = () => {
    // Sanity check the duration.
    // Unnecessary for this buffering, except helps with test coverage.
    var duration = util.mediaSource.duration;
    if (isNaN(duration) || duration <= 0.0) {
      postMessage({
        subject: messageSubject.ERROR,
        info: "mediaSource.duration " + duration + " is not within expected range (0,1)"
      });
      return;
    }

    // Await the main thread media element duration matching the worker
    // mediaSource duration.
    postMessage(getAwaitCurrentDurationRequest());
  };

  util.mediaLoadPromise.then(mediaData => { sourceBuffer.appendBuffer(mediaData); },
                             err => { postMessage({ subject: messageSubject.ERROR, info: err }) });
}


function getAwaitCurrentDurationRequest() {
  // Sanity check that we have a numeric duration value now.
  const dur = util.mediaSource.duration;
  assert(!Number.isNaN(dur), "Unexpected NaN duration in worker");
  return { subject: messageSubject.AWAIT_DURATION, info: dur };
}

function assert(conditionBool, description) {
  if (conditionBool !== true) {
    postMessage({
      subject: messageSubject.ERROR,
      info: "Current test phase [" + phase + "] failed worker assertion. " + description
    });
  }
}

function processPhase(isResponseToAck = false) {
  assert(!isResponseToAck || (phase !== testPhase.kInitial && phase !== testPhase.kAttaching),
      "Phase does not expect verification ack receipt from main thread");

  // Some static request messages useful in transmission and ACK verification.
  const nanDurationCheckRequest = { subject: messageSubject.VERIFY_DURATION, info: NaN };
  const haveNothingReadyStateCheckRequest = { subject: messageSubject.VERIFY_HAVE_NOTHING };
  const setDurationCheckRequest = { subject: messageSubject.AWAIT_DURATION, info: 0.1 };
  const atLeastHaveMetadataReadyStateCheckRequest = { subject: messageSubject.VERIFY_AT_LEAST_HAVE_METADATA };

  switch (phase) {

    case testPhase.kInitial:
      assert(Number.isNaN(util.mediaSource.duration), "Initial unattached MediaSource duration must be NaN, but instead is " + util.mediaSource.duration);
      phase = testPhase.kAttaching;
      let handle = util.mediaSource.handle;
      postMessage({ subject: messageSubject.HANDLE, info: handle }, { transfer: [handle] } );
      break;

    case testPhase.kAttaching:
      postMessage({
        subject: messageSubject.ERROR,
        info: "kAttaching phase is handled by main thread and by worker onsourceopen, not this switch case."
      });
      break;

    case testPhase.kRequestNaNDurationCheck:
      assert(!isResponseToAck);
      postMessage(nanDurationCheckRequest);
      phase = testPhase.kConfirmNaNDurationResult;
      break;

    case testPhase.kConfirmNaNDurationResult:
      assert(isResponseToAck);
      if (checkAckVerificationData(nanDurationCheckRequest)) {
        phase = testPhase.kRequestHaveNothingReadyStateCheck;
        processPhase();
      }
      break;

    case testPhase.kRequestHaveNothingReadyStateCheck:
      assert(!isResponseToAck);
      postMessage(haveNothingReadyStateCheckRequest);
      phase = testPhase.kConfirmHaveNothingReadyStateResult;
      break;

    case testPhase.kConfirmHaveNothingReadyStateResult:
      assert(isResponseToAck);
      if (checkAckVerificationData(haveNothingReadyStateCheckRequest)) {
        phase = testPhase.kRequestSetDurationCheck;
        processPhase();
      }
      break;

    case testPhase.kRequestSetDurationCheck:
      assert(!isResponseToAck);
      const newDuration = setDurationCheckRequest.info;
      assert(!Number.isNaN(newDuration) && newDuration > 0);

      // Set the duration, then request verification.
      util.mediaSource.duration = newDuration;
      postMessage(setDurationCheckRequest);
      phase = testPhase.kConfirmSetDurationResult;
      break;

    case testPhase.kConfirmSetDurationResult:
      assert(isResponseToAck);
      if (checkAckVerificationData(setDurationCheckRequest)) {
        phase = testPhase.kRequestHaveNothingReadyStateRecheck;
        processPhase();
      }
      break;

    case testPhase.kRequestHaveNothingReadyStateRecheck:
      assert(!isResponseToAck);
      postMessage(haveNothingReadyStateCheckRequest);
      phase = testPhase.kConfirmHaveNothingReadyStateRecheckResult;
      break;

    case testPhase.kConfirmHaveNothingReadyStateRecheckResult:
      assert(isResponseToAck);
      if (checkAckVerificationData(haveNothingReadyStateCheckRequest)) {
        phase = testPhase.kRequestAwaitNewDurationCheck;
        processPhase();
      }
      break;

    case testPhase.kRequestAwaitNewDurationCheck:
      assert(!isResponseToAck);
      bufferMediaAndSendDurationVerificationRequest();
      phase = testPhase.kConfirmAwaitNewDurationResult;
      break;

    case testPhase.kConfirmAwaitNewDurationResult:
      assert(isResponseToAck);
      if (checkAckVerificationData(getAwaitCurrentDurationRequest())) {
        phase = testPhase.kRequestAtLeastHaveMetadataReadyStateCheck;
        processPhase();
      }
      break;

    case testPhase.kRequestAtLeastHaveMetadataReadyStateCheck:
      assert(!isResponseToAck);
      postMessage(atLeastHaveMetadataReadyStateCheckRequest);
      phase = testPhase.kConfirmAtLeastHaveMetadataReadyStateResult;
      break;

    case testPhase.kConfirmAtLeastHaveMetadataReadyStateResult:
      assert(isResponseToAck);
      if (checkAckVerificationData(atLeastHaveMetadataReadyStateCheckRequest)) {
        postMessage({ subject: messageSubject.WORKER_DONE });
      }
      phase = "No further phase processing should occur once WORKER_DONE message has been sent";
      break;

    default:
      postMessage({
        subject: messageSubject.ERROR,
        info: "Unexpected test phase in worker:" + phase,
      });
  }

}
