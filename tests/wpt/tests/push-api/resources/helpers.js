function resetSw() {
  return navigator.serviceWorker.getRegistrations().then(registrations => {
    return Promise.all(registrations.map(r => r.unregister()));
  });
}

async function registerSw(path) {
  await resetSw();
  add_completion_callback(resetSw);
  const reg = await navigator.serviceWorker.register(path);
  return reg;
}
