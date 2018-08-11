importScripts('sw-helpers.js');

async function updateUI(event) {
  let updateParams = [];
  switch (event.id) {
    case 'update-once':
      updateParams = [{title: 'Title1'}];
      break;
    case 'update-twice':
      updateParams = [{title: 'Title1'}, {title: 'Title2'}];
      break;
  }

  return Promise.all(updateParams.map(param => event.updateUI(param)))
           .then(() => 'update success')
           .catch(e => e.message);
}

self.addEventListener('backgroundfetched', event => {
  event.waitUntil(updateUI(event)
      .then(update => sendMessageToDocument({ type: event.type, update })))
});
