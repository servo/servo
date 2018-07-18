let source = null;

async function getFetchResult(settledFetch) {
  if (!settledFetch.response)
    return Promise.resolve(null);

  return {
    url: settledFetch.response.url,
    status: settledFetch.response.status,
    text: await settledFetch.response.text(),
  };
}

self.addEventListener('message', event => {
  source = event.source;
  source.postMessage('ready');
});

self.addEventListener('backgroundfetched', event => {
  event.waitUntil(
    event.fetches.values()
      .then(fetches => Promise.all(fetches.map(fetch => getFetchResult(fetch))))
      .then(results => source.postMessage({ type: event.type, results })));
});
