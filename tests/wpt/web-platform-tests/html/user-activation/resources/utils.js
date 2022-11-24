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
