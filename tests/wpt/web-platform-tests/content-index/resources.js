'use strict';

const swUrl = 'resources/sw.js';
const scope = 'resources/';

async function expectTypeError(promise) {
  try {
    await promise;
    assert_unreached('Promise should have rejected');
  } catch (e) {
    assert_equals(e.name, 'TypeError');
  }
}

function createDescription({id = 'id', title = 'title', description = 'description',
                            category = 'homepage', iconUrl = '/images/green-256x256.png',
                            url = scope, includeIcons = true}) {
  return {id, title, description, category, icons: includeIcons ? [{src: iconUrl}] : [], url};
}

// Creates a Promise test for |func| given the |description|. The |func| will be
// executed with the `index` object of an activated Service Worker Registration.
function contentIndexTest(func, description) {
  promise_test(async t => {
    const registration = await service_worker_unregister_and_register(t, swUrl, scope);
    await wait_for_state(t, registration.installing, 'activated');
    return func(t, registration.index);
  }, description);
}

async function waitForMessageFromServiceWorker() {
  return await new Promise(resolve => {
    const listener = event => {
      navigator.serviceWorker.removeEventListener('message', listener);
      resolve(event.data);
    };

    navigator.serviceWorker.addEventListener('message', listener);
  });
}

// Returns a promise if the chromium based browser fetches icons for
// content-index.
async function fetchesIconsChromium() {
  const {fetchesIcons} =
      await import('/resources/chromium/content-index-helpers.js');
  return fetchesIcons();
}

// Returns a promise if the browser fetches icons for content-index and should
// therefore validate them.
async function fetchesIcons() {
  if (isChromiumBased) {
    return fetchesIconsChromium();
  }
  return false;
}
