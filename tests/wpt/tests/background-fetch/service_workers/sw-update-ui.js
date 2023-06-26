importScripts('/resources/testharness.js');
importScripts('sw-helpers.js');

async function updateUI(event) {
  let updateParams = [];
  switch (event.registration.id) {
    case 'update-once':
      updateParams = [{title: 'Title1'}];
      break;
    case 'update-twice':
      updateParams = [{title: 'Title1'}, {title: 'Title2'}];
      break;
  }

  return Promise.all(updateParams.map(param => event.updateUI(param)))
           .then(() => 'update success')
           .catch(e => e.name);
}

self.addEventListener('backgroundfetchsuccess', event => {
  if (event.registration.id === 'update-inactive') {
    // Post an async task before calling updateUI from the inactive event.
    // Any async behaviour outside `waitUntil` should mark the event as
    // inactive, and subsequent calls to `updateUI` should fail.
    new Promise(r => step_timeout(r, 0))
        .then(() => event.updateUI({ title: 'New title' }))
        .catch(e => sendMessageToDocument({ update: e.name }));
    return;
  }

  event.waitUntil(updateUI(event)
      .then(update => sendMessageToDocument({ update })));
});
