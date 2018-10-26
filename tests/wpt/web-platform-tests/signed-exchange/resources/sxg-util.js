// Opens |url| in an iframe, establish a message channel with it, and waits for
// a message from the frame content. Returns a promise that resolves with the
// data of the message, or rejects on 2000ms timeout.
function openSXGInIframeAndWaitForMessage(test_object, url) {
  return new Promise(async (resolve, reject) => {
    // We can't catch the network error on iframe. So we use the timer.
    test_object.step_timeout(() => reject('timeout'), 2000);

    const frame = await withIframe(url, 'sxg_iframe');
    const channel = new MessageChannel();
    channel.port1.onmessage = (event) => resolve(event.data);
    frame.contentWindow.postMessage(
        {port: channel.port2}, '*', [channel.port2]);
  });
}

function withIframe(url, name) {
  return new Promise((resolve, reject) => {
      const frame = document.createElement('iframe');
      frame.src = url;
      frame.name = name;
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
