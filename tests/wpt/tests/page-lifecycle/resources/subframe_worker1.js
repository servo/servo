var bc = new BroadcastChannel('subworker_channel');

setInterval(() => {
  bc.postMessage('subworker');
}, 10);

w2 = new Worker("subframe_worker2.js");
