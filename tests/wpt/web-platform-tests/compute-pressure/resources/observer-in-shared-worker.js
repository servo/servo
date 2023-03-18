'use strict';

onconnect = function(e) {
  const port = e.ports[0];  // get the port
  let started = false;

  port.start();  // Open the port connection to enable two-way communication

  let observerWorker =
      new PressureObserver(pressureCallback, {sampleRate: 0.5});

  port.onmessage = function(e) {
    if (started === false)
      observerWorker.observe('cpu');
    started = true;
  };

  function pressureCallback(update) {
    port.postMessage(update[0].toJSON());
  };
}
