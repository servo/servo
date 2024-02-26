function unregisterAllServiceWorker() {
  return navigator.serviceWorker.getRegistrations().then(registrations => {
    return Promise.all(registrations.map(r => r.unregister()));
  });
}

async function getActiveServiceWorker(script) {
  await unregisterAllServiceWorker();
  const reg = await navigator.serviceWorker.register(script);
  add_completion_callback(() => reg.unregister());
  await navigator.serviceWorker.ready;
  return reg;
}


async function closeAllNotifications() {
  for (const n of await registration.getNotifications()) {
    n.close();
  }
}
