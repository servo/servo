// This test reproduces Chromium issue https://crbug.com/1292387. If it doesn't
// crash then the test passed.

test(() => {
  const iframeTag = document.createElement('iframe');
  document.body.appendChild(iframeTag);
  const wt = new iframeTag.contentWindow.WebTransport('https://example.com/');
  iframeTag.remove();
  const datagrams = wt.datagrams;
  const reader = datagrams.readable;
  reader.cancel();
}, 'call cancel() on stream in destroyed realm');
