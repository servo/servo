'use strict';

function pressureCallback(update) {
  postMessage(update[0].toJSON());
};

const observerWorker =
    new PressureObserver(pressureCallback, {sampleRate: 0.5});

observerWorker.observe('cpu');
