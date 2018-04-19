promise_test(t => {
  const mediaSource = new MediaSource(),
        mediaSourceURL = URL.createObjectURL(mediaSource);
  return promise_rejects(t, new TypeError(), fetch(mediaSourceURL));
}, "Cannot fetch blob: URL from a MediaSource");
