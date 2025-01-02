async function doFetch(url) {
  const response = await fetch(url);
  const body = await response.text();
  return {
    status: response.status,
    body,
  };
}

async function fetchAndPost(url, port) {
  try {
    const message = await doFetch(url);
    port.postMessage(message);
  } catch(e) {
    port.postMessage({ error: e.name });
  }
}

const url = new URL(self.location.href).searchParams.get("url");

self.addEventListener("connect", async (evt) => {
  await fetchAndPost(url, evt.ports[0]);
});
