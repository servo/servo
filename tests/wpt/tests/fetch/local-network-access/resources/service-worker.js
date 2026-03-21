async function doFetch(url) {
  const response = await fetch(url);
  const body = await response.text();
  return {
    ok: response.ok,
    error: response.error,
    body,
  };
}

async function fetchAndPost(url, port) {
  try {
    const message = await doFetch(url);
    port.postMessage(message);
  } catch (e) {
    port.postMessage({error: e.toString()});
  }
}

let webtransport;

async function doWebTransport(url, port) {
  try {
    webtransport = new WebTransport(url);
    await webtransport.ready;
    port.postMessage('open');
  } catch (error) {
    port.postMessage('error');
  }
}

let websocket;

async function doWebSocket(url, port) {
  websocket = new WebSocket(url);
  websocket.onopen = () => {
    port.postMessage('open');
  };

  websocket.onclose = (evt) => {
    port.postMessage(`close: code ${evt.code}`);
  };
}

async function doNavigate(url, id, port) {
  try {
    let targetClient = null;
    const clients = await self.clients.matchAll(
        {type: 'window', includeUncontrolled: true});
    for (const c of clients) {
      const client_url = new URL(c.url);
      if (client_url.searchParams.get('id') == id) {
        targetClient = c;
        break;
      }
    }

    if (targetClient) {
      try {
        targetClient.navigate(url);
        // No postmessage for navigation; service worker does not tell us if
        // navigation succeeds or fails.
      } catch (e) {
        port.postMessage(`failure navigate ${e}`);
      }
    } else {
      port.postMessage('no client');
    }
  } catch (e) {
    port.postMessage(`failure finding client${e}`);
  }
}

async function handleMessage(e) {
  const port = e.ports[0];
  switch (e.data.method) {
    case 'fetch':
      fetchAndPost(e.data.url, port);
      break;
    case 'websocket':
      doWebSocket(e.data.url, port);
      break;
    case 'webtransport':
      doWebTransport(e.data.url, port);
      break;
    case 'navigate':
      doNavigate(e.data.url, e.data.id, port);
  }
};

self.addEventListener('activate', function(event) {
  event.waitUntil(clients.claim());
});

self.addEventListener('message', e => {
  e.waitUntil(handleMessage(e));
});
