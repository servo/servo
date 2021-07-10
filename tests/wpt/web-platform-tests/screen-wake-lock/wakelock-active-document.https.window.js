function getWakeLockObject(iframe, url) {
  return new Promise(resolve => {
    iframe.addEventListener(
      "load",
      () => {
        const { wakeLock } = iframe.contentWindow.navigator;
        resolve(wakeLock);
      },
      { once: true }
    );
    iframe.src = url;
  });
}

promise_test(async t => {
  const iframe = document.createElement("iframe");
  document.body.appendChild(iframe);
  // We first got to page1.html, grab a WakeLock object.
  const wakeLock1 = await getWakeLockObject(
    iframe,
    "/screen-wake-lock/resources/page1.html"
  );
  // Save the DOMException of page1.html before navigating away.
  const frameDOMException1 = iframe.contentWindow.DOMException;
  // We navigate the iframe again, putting wakeLock1's document into an inactive state.
  const wakeLock2 = await getWakeLockObject(
    iframe,
    "/screen-wake-lock/resources/page2.html"
  );
  // Now, wakeLock1's relevant global object's document is no longer active.
  // So, call .request(), and make sure it rejects appropriately.
  await promise_rejects_dom(
    t,
    "NotAllowedError",
    frameDOMException1,
    wakeLock1.request('screen'),
    "Inactive document, so must throw NotAllowedError"
  );
  // We are done, so clean up.
  iframe.remove();
}, "navigator.wakeLock.request() aborts if the document is not active.");

promise_test(async t => {
  // We nest two iframes and wait for them to load.
  const outerIframe = document.createElement("iframe");
  document.body.appendChild(outerIframe);
  // Load the outer iframe (we don't care about the awaited request)
  await getWakeLockObject(
    outerIframe,
    "/screen-wake-lock/resources/page1.html"
  );

  // Now we create the inner iframe
  const innerIframe = outerIframe.contentDocument.createElement("iframe");

  // nest them
  outerIframe.contentDocument.body.appendChild(innerIframe);

  // load innerIframe, and get the WakeLock instance
  const wakeLock = await getWakeLockObject(
    innerIframe,
    "/screen-wake-lock/resources/page2.html"
  );
  // Save DOMException from innerIframe before navigating away.
  const innerIframeDOMException = innerIframe.contentWindow.DOMException;

  // Navigate the outer iframe to a new location.
  // Wait for the load event to fire.
  await new Promise(resolve => {
    outerIframe.addEventListener("load", resolve);
    outerIframe.src = "/screen-wake-lock/resources/page2.html";
  });

  // Now, request's relevant global object's document is still active
  // (it is the active document of the inner iframe), but is not fully active
  // (since the parent of the inner iframe is itself no longer active).
  // So, call request.show() and make sure it rejects appropriately.
  await promise_rejects_dom(
    t,
    "NotAllowedError",
    innerIframeDOMException,
    wakeLock.request('screen'),
    "Active, but not fully active, so must throw NotAllowedError"
  );
  // We are done, so clean up.
  outerIframe.remove();
}, "navigator.wakeLock.request() aborts if the document is active, but not fully active.");
