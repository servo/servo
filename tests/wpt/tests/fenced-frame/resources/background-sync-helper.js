const getOneShotSyncPromise = (registration, method) => {
  if (method === 'register') {
    return registration.sync.register('fencedframe-oneshot');
  } else if (method === 'getTags') {
    return registration.sync.getTags();
  }
  return Promise.resolve();
};

const getPeriodicSyncPromise = (registration, method) => {
  if (method === 'register') {
    return registration.periodicSync.register(
        'fencedframe-periodic', {minInterval: 1000});
  } else if (method === 'getTags') {
    return registration.periodicSync.getTags();
  } else if (method === 'unregister') {
    return registration.periodicSync.unregister('fencedframe-periodic');
  } else {
    return Promise.resolve();
  }
};

export {getOneShotSyncPromise, getPeriodicSyncPromise}
