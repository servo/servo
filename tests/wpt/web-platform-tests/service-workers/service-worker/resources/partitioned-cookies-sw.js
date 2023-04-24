self.addEventListener('message', ev => ev.waitUntil(onMessage(ev)));

async function onMessage(event) {
  if (!event.data)
    return;
  switch (event.data.type) {
    case 'test_message':
      return onTestMessage(event);
    case 'echo_cookies':
      return onEchoCookies(event);
    default:
      return;
  }
}

// test_message just verifies that the message passing is working.
async function onTestMessage(event) {
  event.source.postMessage({ok: true});
}

// echo_cookies returns the names of all of the cookies available to the worker.
async function onEchoCookies(event) {
  try {
    const cookie_objects = await self.cookieStore.getAll();
    const cookies = cookie_objects.map(c => c.name);
    event.source.postMessage({ok: true, cookies});
  } catch (err) {
    event.source.postMessage({ok: false});
  }
}
