function delayByFrames(f, num_frames) {
  function recurse(depth) {
    if (depth == 0)
      f();
    else
      requestAnimationFrame(() => recurse(depth-1));
  }
  recurse(num_frames);
}

// Returns a Promise which is resolved with the event object when the event is
// fired.
function getEvent(eventType) {
  return new Promise(resolve => {
    document.body.addEventListener(eventType, e => resolve(e), {once: true});
  });
}


// Returns a Promise which is resolved with a "true" iff transient activation
// was available and successfully consumed.
//
// This function relies on Fullscreen API to check/consume user activation
// state.
async function consumeTransientActivation() {
  try {
    await document.body.requestFullscreen();
    await document.exitFullscreen();
    return true;
  } catch(e) {
    return false;
  }
}

// Returns a `Promise` that gets resolved when `window` receives a "message"
// event with an `event.data` JSON string whose "type" field matches the given
// parameter.  The promise is resolved with JSON-parsed `event.data`.
function receiveMessage(type) {
  return new Promise((resolve) => {
    window.addEventListener("message", function listener(event) {
      if (typeof event.data !== "string") {
        return;
      }
      const data = JSON.parse(event.data);
      if (data.type === type) {
        window.removeEventListener("message", listener);
        resolve(data);
      }
    });
  });
}

// Helpers for the sticky activation carry-over tests. The two that read the
// state are sent to an executor by RemoteContext.execute_script(), which
// serializes them, so they have to stand on their own; pageShowPromise is
// defined by the back/forward cache executor they run in.
const getUserActivationState = () =>
  [navigator.userActivation.isActive, navigator.userActivation.hasBeenActive];

const getUserActivationStateWhenShown = async () => {
  await window.pageShowPromise;
  return [navigator.userActivation.isActive,
          navigator.userActivation.hasBeenActive];
};

const waitUntilShown = () => window.pageShowPromise;

function assertUserActivationState(list,
      expected_transient_bit, expected_sticky_bit, msg) {
  assert_equals(list[0], expected_transient_bit, "Transient bit " + msg);
  assert_equals(list[1], expected_sticky_bit, "Sticky bit " + msg);
}
