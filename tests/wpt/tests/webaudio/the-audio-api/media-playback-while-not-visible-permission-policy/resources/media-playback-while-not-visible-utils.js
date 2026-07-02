// Shared helpers for the media-playback-while-not-visible permission policy
// tests. The same helpers drive both the same-origin and cross-origin
// scenarios; the only difference between the two is the origin of the iframe
// that hosts the AudioContext, which is passed in as `base` (a URL prefix to
// the resources/ directory on the desired origin).

// Returns a promise that resolves when the iframe's AudioContext emits a
// 'statechange' event. The promise resolves with the new state of the
// AudioContext, or with 'no state change' if no event fires within `timeout`
// milliseconds. The event listener is removed once the promise settles.
// Callers that expect no state change can pass a shorter `timeout` to avoid
// waiting the full default duration on every negative assertion.
function expectIframeAudioContextStateChangeEvent(test, timeout = 2000) {
  return new Promise(resolve => {
    function expectStateChangeEvent(event) {
      if (event.data.operation !== 'statechange') {
        return;
      }
      window.removeEventListener('message', expectStateChangeEvent);
      resolve(event.data.value);
    }

    window.addEventListener('message', expectStateChangeEvent);
    test.step_timeout(() => {
      window.removeEventListener('message', expectStateChangeEvent);
      resolve('no state change');
    }, timeout);
  });
}

// Sends a message to the iframe to query the state of the AudioContext and
// returns a promise that resolves with the state of the AudioContext. The
// event listener is removed after the promise resolves.
function queryAudioContextState(iframe) {
  return new Promise(resolve => {
    window.addEventListener(
        'message', function expectStateQueryResponse(event) {
          if (event.data.operation !== 'getState') {
            return;
          }
          window.removeEventListener('message', expectStateQueryResponse);
          resolve(event.data.value);
        });
    iframe.contentWindow.postMessage('getState', '*');
  });
}

// Sends a message to the iframe with a command that should be performed on
// the AudioContext. Returns a promise that resolves when the operation is
// completed. The event listener is removed after the promise resolves.
function sendCommandToAudioContext(iframe, command) {
  return new Promise(resolve => {
    window.addEventListener('message', function expectCommandResponse(event) {
      if (event.data.operation !== command) {
        return;
      }
      window.removeEventListener('message', expectCommandResponse);
      resolve(event.data.value);
    });
    iframe.contentWindow.postMessage(command, '*');
  });
}

// Drives the iframe's AudioContext into `expected_initial_state` ('running' or
// 'suspended'). Returns a promise that resolves with the resulting state.
async function setUpIframeAudioContextInitialState(t, iframe,
                                                   expected_initial_state) {
  const initial_state = await queryAudioContextState(iframe);
  if (initial_state === expected_initial_state || initial_state === 'closed') {
    return initial_state;
  }

  const statechange_promise = expectIframeAudioContextStateChangeEvent(t);
  if (expected_initial_state === 'running') {
    await sendCommandToAudioContext(iframe, 'resume');
  } else if (expected_initial_state === 'suspended') {
    await sendCommandToAudioContext(iframe, 'suspend');
  }

  return statechange_promise;
}

function hideFrame(iframe, type) {
  if (type === 'display') {
    iframe.style.setProperty('display', 'none');
  } else if (type === 'visibility') {
    iframe.style.setProperty('visibility', 'hidden');
  } else if (type === 'zero-size') {
    iframe.style.setProperty('width', '0');
    iframe.style.setProperty('height', '0');
  }
}

function showFrame(iframe, type) {
  if (type === 'display') {
    iframe.style.setProperty('display', 'block');
  } else if (type === 'visibility') {
    iframe.style.setProperty('visibility', 'visible');
  } else if (type === 'zero-size') {
    iframe.style.removeProperty('width');
    iframe.style.removeProperty('height');
  }
}

// Creates an iframe that hosts an AudioContext and waits for it to reach the
// 'running' state. `base` is the URL prefix (including trailing slash) to the
// resources/ directory on the desired origin; using the alternate-host base
// produces a genuine cross-origin (out-of-process) iframe. If `frameType` is
// 'nested', an intermediate iframe is inserted between the test page and the
// AudioContext frame, so the test page and the AudioContext frame are always
// separated by at least one frame boundary on origin `base`.
async function createIframe(t, frameType, base) {
  if (document.readyState !== 'complete') {
    await new Promise(resolve => window.addEventListener('load', resolve));
  }

  const running_state_transition_promise =
      expectIframeAudioContextStateChangeEvent(t);

  const iframe = document.createElement('iframe');
  if (frameType === 'nested') {
    iframe.id = 'intermediate-frame';
    iframe.src = base + 'intermediate-frame.html';
  } else {
    iframe.id = 'audio-context-frame';
    iframe.allow = 'media-playback-while-not-visible \'none\'; autoplay *';
    iframe.src = base + 'audiocontext-frame.html';
  }

  document.body.appendChild(iframe);
  await new Promise(resolve => iframe.addEventListener('load', resolve));
  t.add_cleanup(() => iframe.remove());

  assert_equals(await running_state_transition_promise, 'running');
  return iframe;
}
