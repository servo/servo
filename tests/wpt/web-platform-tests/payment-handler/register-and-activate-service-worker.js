async function registerAndActiveServiceWorker(script, scope, callback) {
  const registration = await navigator.serviceWorker.register(script, {scope});
  const serviceWorker =
    registration.installing || registration.waiting || registration.active;
  if (serviceWorker) {
    waitForServiceWorkerActivation(scope, callback);
    return;
  }

  registration.addEventListener('updatefound', event => {
    waitForServiceWorkerActivation(scope, callback);
  });
}

async function waitForServiceWorkerActivation(scope, callback) {
  const registration = await navigator.serviceWorker.getRegistration(scope);
  if (registration.active) {
    callback(registration);
    return;
  }

  const serviceWorker = registration.installing || registration.waiting;
  serviceWorker.addEventListener('statechange', event => {
    if (event.target.state == 'activated') {
      callback(registration);
    }
  });
}
