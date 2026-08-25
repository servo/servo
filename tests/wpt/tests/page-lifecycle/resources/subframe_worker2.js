var bc = new BroadcastChannel('subworker_channel');

setInterval(() => {
  bc.postMessage('subworker2');
}, 10);
