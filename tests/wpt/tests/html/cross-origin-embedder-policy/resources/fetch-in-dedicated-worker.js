self.addEventListener('message', async (e) => {
  const param = e.data;
  // Ignore network error.
  await fetch(param.url, param.init).catch(() => {});
  self.postMessage(param.url);
});
