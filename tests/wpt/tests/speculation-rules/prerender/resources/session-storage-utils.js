function getSessionStorageKeys() {
  let keys = [];
  let txt = '';
  for (let i = 0; i < sessionStorage.length; ++i) {
    keys.push(sessionStorage.key(i));
  }
  keys.sort();
  keys.forEach((key) => {
    if (txt.length) {
      txt += ', ';
    }
    txt += key;
  });
  return txt;
}

function getNextMessage(channel) {
  return new Promise(resolve => {
    channel.addEventListener('message', e => {
      resolve(e.data);
    }, {once: true});
  });
}

// session_storage_test() is a utility function for running session storage
// related tests that open a initiator page using window.open().
function session_storage_test(testPath, uid) {
  promise_test(async t => {
    const testChannel = new PrerenderChannel('test-channel', uid);
    t.add_cleanup(() => {
      testChannel.close();
    });
    const gotMessage = getNextMessage(testChannel);
    const url = 'resources/' + testPath + '?uid=' + uid;
    window.open(url, '_blank', 'noopener');
    assert_equals(await gotMessage, 'Done');
  }, testPath);
}

// RunSessionStorageTest() is a utility function for running session storage
// related tests that requires coordinated code execution on both the initiator
// page and the prerendering page. The passed |func| function will be called
// with the following arguments:
//   - isPrerendering: Whether the |func| is called in the prerendering page.
//   - url: The URL of the prerendering page. |func| should call
//     startPrerendering(url) when |isPrerendering| is false to start the
//     prerendering.
//   - channel: A PrerenderChannel which can be used to coordinate the code
//     execution on the initiator page and the prerendering page.
//   - done: A function that should be called when the test completes
//     successfully.
async function RunSessionStorageTest(func, uid) {
  const url = new URL(document.URL);
  url.searchParams.set('prerendering', '');
  const params = new URLSearchParams(location.search);
  // The main test page loads the initiator page, then the initiator page will
  // prerender itself with the `prerendering` parameter.
  const isPrerendering = params.has('prerendering');
  const prerenderChannel = new PrerenderChannel('prerender-channel', uid);
  const testChannel = new PrerenderChannel('test-channel', uid);
  window.addEventListener('pagehide', () => {
    prerenderChannel.close();
    testChannel.close();
  });
  try {
    await func(isPrerendering, url.toString(), prerenderChannel, () => {
      testChannel.postMessage('Done');
    })
  } catch (e) {
    testChannel.postMessage(e.toString());
  }
}
