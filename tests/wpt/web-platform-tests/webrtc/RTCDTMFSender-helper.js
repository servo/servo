'use strict';

// Test is based on the following editor draft:
// https://w3c.github.io/webrtc-pc/archives/20170605/webrtc.html

// Code using this helper should also include RTCPeerConnection-helper.js
// in the main HTML file

// The following helper functions are called from RTCPeerConnection-helper.js:
//   getTrackFromUserMedia

// Create a RTCDTMFSender using getUserMedia()
function createDtmfSender(pc = new RTCPeerConnection()) {
  return getTrackFromUserMedia('audio')
  .then(([track, mediaStream]) => {
    const sender = pc.addTrack(track, mediaStream);
    const dtmfSender = sender.dtmf;

    assert_true(dtmfSender instanceof RTCDTMFSender,
      'Expect audio sender.dtmf to be set to a RTCDTMFSender');

    return dtmfSender;
  });
}

/*
  Create an RTCDTMFSender and test tonechange events on it.
    testFunc
      Test function that is going to manipulate the DTMFSender.
      It will be called with:
        t - the test object
        sender - the created RTCDTMFSender
        pc - the associated RTCPeerConnection as second argument.
    toneChanges
      Array of expected tonechange events fired. The elements
      are array of 3 items:
        expectedTone
          The expected character in event.tone
        expectedToneBuffer
          The expected new value of dtmfSender.toneBuffer
        expectedDuration
          The rough time since beginning or last tonechange event
          was fired.
    desc
      Test description.
 */
function test_tone_change_events(testFunc, toneChanges, desc) {
  async_test(t => {
    const pc = new RTCPeerConnection();

    createDtmfSender(pc)
    .then(dtmfSender => {
      let lastEventTime = Date.now();

      const onToneChange = t.step_func(ev => {
        assert_true(ev instanceof RTCDTMFToneChangeEvent,
          'Expect tone change event object to be an RTCDTMFToneChangeEvent');

        const { tone } = ev;
        assert_equals(typeof tone, 'string',
          'Expect event.tone to be the tone string');

        assert_greater_than(toneChanges.length, 0,
          'More tonechange event is fired than expected');

        const [
          expectedTone, expectedToneBuffer, expectedDuration
        ] = toneChanges.shift();

        assert_equals(tone, expectedTone,
          `Expect current event.tone to be ${expectedTone}`);

        assert_equals(dtmfSender.toneBuffer, expectedToneBuffer,
          `Expect dtmfSender.toneBuffer to be updated to ${expectedToneBuffer}`);

        const now = Date.now();
        const duration = now - lastEventTime;

        assert_approx_equals(duration, expectedDuration, 250,
          `Expect tonechange event for "${tone}" to be fired approximately after ${expectedDuration} milliseconds`);

        lastEventTime = now;

        if(toneChanges.length === 0) {
          // Wait for same duration as last expected duration + 100ms
          // before passing test in case there are new tone events fired,
          // in which case the test should fail.
          t.step_timeout(
            t.step_func(() => {
              t.done();
              pc.close();
            }), expectedDuration + 100);
        }
      });

      dtmfSender.addEventListener('tonechange', onToneChange);

      testFunc(t, dtmfSender, pc);
    })
    .catch(t.step_func(err => {
      assert_unreached(`Unexpected promise rejection: ${err}`);
    }));
  }, desc);
}

// Get the one and only tranceiver from pc.getTransceivers().
// Assumes that there is only one tranceiver in pc.
function getTransceiver(pc) {
  const transceivers = pc.getTransceivers();
  assert_equals(transceivers.length, 1,
    'Expect there to be only one tranceiver in pc');

  return transceivers[0];
}
