// Opens |url| in an iframe, establish a message channel with it, and waits for
// a message from the frame content. Returns a promise that resolves with the
// data of the message, or rejects on 3000ms timeout.
function openSXGInIframeAndWaitForMessage(test_object, url, referrerPolicy) {
  return new Promise(async (resolve, reject) => {
    // We can't catch the network error on iframe. So we use the timer.
    test_object.step_timeout(() => reject('timeout'), 3000);

    const frame = await withIframe(url, 'sxg_iframe', referrerPolicy);
    const channel = new MessageChannel();
    channel.port1.onmessage = (event) => resolve(event.data);
    frame.contentWindow.postMessage(
        {port: channel.port2}, '*', [channel.port2]);
  });
}

function withIframe(url, name, referrerPolicy) {
  return new Promise((resolve, reject) => {
      const frame = document.createElement('iframe');
      frame.src = url;
      frame.name = name;
      if (referrerPolicy !== undefined) {
        frame.referrerPolicy = referrerPolicy;
      }
      frame.onload = () => resolve(frame);
      frame.onerror = () => reject('failed to load ' + url);
      document.body.appendChild(frame);
    });
}

function loadScript(url) {
  return new Promise((resolve, reject) => {
    const scriptTag = document.createElement('script');
    scriptTag.src = url;
    scriptTag.onload = () => resolve();
    scriptTag.onerror = () => reject('failed to load ' + url);
    document.head.appendChild(scriptTag);
  });
}

function innerURLOrigin() {
  return 'https://127.0.0.1:8444';
}

function runReferrerTests(test_cases) {
  for (const i in test_cases) {
    const test_case = test_cases[i];
    promise_test(async (t) => {
      const sxgUrl = test_case.origin + '/signed-exchange/resources/sxg/' +
                     test_case.sxg;
      const message =
          await openSXGInIframeAndWaitForMessage(
              t, sxgUrl, test_case.referrerPolicy);
      assert_false(message.is_fallback);
      assert_equals(message.referrer, test_case.expectedReferrer);

      const invalidSxgUrl =
          test_case.origin + '/signed-exchange/resources/sxg/invalid-' +
          test_case.sxg;
      const fallbackMessage =
          await openSXGInIframeAndWaitForMessage(
                t, invalidSxgUrl, test_case.referrerPolicy);
      assert_true(fallbackMessage.is_fallback);
      assert_equals(fallbackMessage.referrer, test_case.expectedReferrer);
    }, 'Referrer of SignedHTTPExchange test : ' + JSON.stringify(test_case));
  }
}

function addPrefetch(url) {
  const link = document.createElement('link');
  link.rel = 'prefetch';
  link.href = url;
  document.body.appendChild(link);
}
