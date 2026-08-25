// Shared helpers for the media-playback-while-not-visible permission policy
// tests. The same helpers drive both the same-origin and cross-origin
// scenarios; the only difference between the two is the origin of the iframe
// that hosts the media element, which is passed in as `base` (a URL prefix to
// the resources/ directory on the desired origin).
function queryPlayerStatus(iframe) {
  return new Promise(resolve => {
    window.addEventListener('message', function handler(event) {
      if (event.data.type === 'queryPlayerStatus') {
        window.removeEventListener('message', handler);
        resolve(event.data.status);
      }
    });
    iframe.contentWindow.postMessage({action: 'queryPlayerStatus'}, '*');
  });
}

// Sends a message to the iframe asking it to play its media element. Returns a
// promise that resolves with 'Success' if playback started, or with the name
// of the DOMException (e.g. 'NotAllowedError') if play() was rejected.
function playMediaInIframe(iframe) {
  return new Promise(resolve => {
    window.addEventListener('message', function handler(event) {
      if (event.data.type === 'play') {
        window.removeEventListener('message', handler);
        resolve(event.data.status);
      }
    });
    iframe.contentWindow.postMessage({action: 'play'}, '*');
  });
}

// Sends a message to the iframe asking it to pause its media element. Returns a
// promise that resolves with 'Success' once the media element is paused.
function pauseMediaInIframe(iframe) {
  return new Promise(resolve => {
    window.addEventListener('message', function handler(event) {
      if (event.data.type !== 'pause') {
        return;
      }
      window.removeEventListener('message', handler);
      resolve(event.data.status);
    });
    iframe.contentWindow.postMessage({action: 'pause'}, '*');
  });
}

// Returns a promise that resolves when the iframe's media element emits a
// playback 'statechange' message. The promise resolves with the new state or
// with 'no state change' if no message fires within `timeout` milliseconds. The
// event listener is removed once the promise settles. Callers that expect no
// state change can pass a shorter `timeout` to avoid waiting the full default
// duration on every negative assertion.
function expectMediaPlayerStateChangeInIframe(test, timeout = 2000) {
  return new Promise(resolve => {
    function handler(event) {
      if (event.data.type === 'statechange') {
        window.removeEventListener('message', handler);
        resolve(event.data.newState);
      }
    }

    window.addEventListener('message', handler);
    test.step_timeout(() => {
      window.removeEventListener('message', handler);
      resolve('no state change');
    }, timeout);
  });
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

// Polls the media frame until it reports that it has finished loading. The
// frame answers 'queryIsLoaded' messages with its current readiness; each poll
// re-sends the query (so a query sent before the frame installed its handler is
// simply retried) and returns the latest response.
async function waitForMediaFrameLoaded(t, iframe) {
  let isLoaded = false;
  function onQueryIsLoadedResponse(event) {
    if (event.data && event.data.type === 'queryIsLoaded') {
      isLoaded = event.data.isLoaded;
    }
  }
  window.addEventListener('message', onQueryIsLoadedResponse);
  await t.step_wait(() => {
    iframe.contentWindow.postMessage({action: 'queryIsLoaded'}, '*');
    return isLoaded;
  }, 'waiting for the media frame to finish loading');
  window.removeEventListener('message', onQueryIsLoadedResponse);
}

// Creates an iframe that hosts a media element. `base` is the URL prefix
// (including trailing slash) to the resources/ directory on the desired origin;
// using the alternate-host base produces a genuine cross-origin
// (out-of-process) iframe. `mediaType` selects the media element type ('video'
// or 'audio'). If `frameType` is 'nested', an intermediate iframe is inserted
// between the test page and the media element frame, so the test page and the
// media element frame are always separated by at least one frame boundary on
// origin `base`.
async function createMediaIframe(t, frameType, base, mediaType) {
  if (document.readyState !== 'complete') {
    await new Promise(resolve => window.addEventListener('load', resolve));
  }

  const iframe = document.createElement('iframe');
  if (frameType === 'nested') {
    iframe.id = 'intermediate-frame';
    iframe.src = base + 'intermediate-frame.html?media=' + mediaType;
  } else {
    iframe.id = 'media-frame';
    iframe.allow = 'media-playback-while-not-visible \'none\'; autoplay *';
    iframe.src = base + 'media-frame.html?media=' + mediaType;
  }

  document.body.appendChild(iframe);
  await waitForMediaFrameLoaded(t, iframe);

  t.add_cleanup(() => iframe.remove());
  return iframe;
}

// Creates a media element iframe and ensures the media element is paused.
async function createMediaIframeAndPause(t, frameType, base, mediaType) {
  const iframe = await createMediaIframe(t, frameType, base, mediaType);
  assert_equals(await pauseMediaInIframe(iframe), 'Success');
  return iframe;
}
