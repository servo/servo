async function doFetch(url) {
  const response = await fetch(url);
  const body = await response.text();
  return {
    ok: response.ok,
    error: response.error,
    body,
  };
}

async function fetchAndPost(url) {
  try {
    const message = await doFetch(url);
    self.postMessage(message);
  } catch (e) {
    self.postMessage({error: e.toString()});
  }
}

let webtransport;

async function doWebTransport(url) {
  try {
    webtransport = new WebTransport(url);
    await webtransport.ready;
    self.postMessage('open');
  } catch (error) {
    self.postMessage('error');
  }
}

let websocket;

async function doWebSocket(url) {
  websocket = new WebSocket(url);
  websocket.onopen = () => {
    self.postMessage('open');
  };

  websocket.onclose = (evt) => {
    self.postMessage(`close: code ${evt.code}`);
  };
}

self.onmessage = (e) => {
  switch (e.data.method) {
    case 'fetch':
      fetchAndPost(e.data.url);
      break;
    case 'websocket':
      doWebSocket(e.data.url);
      break;
    case 'webtransport':
      doWebTransport(e.data.url);
      break;
  }
}
