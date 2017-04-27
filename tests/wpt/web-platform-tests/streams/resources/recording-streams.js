'use strict';

self.recordingReadableStream = (extras = {}, strategy) => {
  let controllerToCopyOver;
  const stream = new ReadableStream({
    start(controller) {
      controllerToCopyOver = controller;

      if (extras.start) {
        return extras.start(controller);
      }

      return undefined;
    },
    pull(controller) {
      stream.events.push('pull');

      if (extras.pull) {
        return extras.pull(controller);
      }

      return undefined;
    },
    cancel(reason) {
      stream.events.push('cancel', reason);
      stream.eventsWithoutPulls.push('cancel', reason);

      if (extras.cancel) {
        return extras.cancel(reason);
      }

      return undefined;
    }
  }, strategy);

  stream.controller = controllerToCopyOver;
  stream.events = [];
  stream.eventsWithoutPulls = [];

  return stream;
};

self.recordingWritableStream = (extras = {}, strategy) => {
  let controllerToCopyOver;
  const stream = new WritableStream({
    start(controller) {
      controllerToCopyOver = controller;

      if (extras.start) {
        return extras.start(controller);
      }

      return undefined;
    },
    write(chunk, controller) {
      stream.events.push('write', chunk);

      if (extras.write) {
        return extras.write(chunk, controller);
      }

      return undefined;
    },
    close() {
      stream.events.push('close');

      if (extras.close) {
        return extras.close();
      }

      return undefined;
    },
    abort(e) {
      stream.events.push('abort', e);

      if (extras.abort) {
        return extras.abort(e);
      }

      return undefined;
    }
  }, strategy);

  stream.controller = controllerToCopyOver;
  stream.events = [];

  return stream;
};
