"use strict";

function describeTrack(track) {
  return {
    id: track.id,
    kind: track.kind,
    label: track.label,
    readyState: track.readyState,
    enabled: track.enabled,
    muted: track.muted,
    contentHint: track.contentHint,
    settings: track.getSettings(),
    constraints: track.getConstraints(),
  };
}

// Prepended to every worker script. Each message is dispatched to
// handlers[data.type] and the return value is posted back as {result}, or
// {error} if the handler threw. A handler returns transfer(value, list) to
// transfer objects along with its reply.
const WORKER_PRELUDE = describeTrack.toString() + `
  const handlers = {};
  const eventLogs = new Map();

  function logEvents(target, types) {
    const log = [];
    eventLogs.set(target, log);
    for (const type of types) {
      target.addEventListener(type, () => log.push(type));
    }
  }

  async function waitForEvent(target, type) {
    const log = eventLogs.get(target);
    if (!log.includes(type)) {
      await new Promise(r => target.addEventListener(type, r, {once: true}));
    }
    log.splice(log.indexOf(type), 1);
  }

  function takeEvents(target) {
    const log = eventLogs.get(target);
    return log.splice(0, log.length);
  }

  async function readFrame(track) {
    const reader =
      new MediaStreamTrackProcessor({track}).readable.getReader();
    const {value, done} = await reader.read();
    if (done) {
      throw new Error("Stream ended before a frame was produced");
    }
    const size = {width: value.codedWidth, height: value.codedHeight};
    value.close();
    await reader.cancel();
    return size;
  }

  function transfer(result, list) {
    return {__transfer: list, result};
  }

  self.onmessage = async e => {
    try {
      const r = await handlers[e.data.type](e.data);
      if (r && r.__transfer) {
        self.postMessage({result: r.result}, r.__transfer);
      } else {
        self.postMessage({result: r});
      }
    } catch (err) {
      self.postMessage({error: err.name + ": " + err.message});
    }
  };
`;

// Handlers operating on a single track received with the "receive" message.
const TRACK_WORKER_SCRIPT = `
  handlers.receive = ({track}) => {
    self.track = track;
    logEvents(track, ["ended", "mute", "unmute"]);
    return describeTrack(track);
  };
  handlers.state = () => describeTrack(self.track);
  handlers.events = () => takeEvents(self.track);
  handlers.readFrame = () => readFrame(self.track);
  handlers.waitForEvent = ({name}) => waitForEvent(self.track, name);
  handlers.stop = () => {
    self.track.stop();
    return describeTrack(self.track);
  };
  handlers.noop = () => {};
`;

async function createWorker(t, script) {
  const blob = new Blob([WORKER_PRELUDE, script, "self.postMessage('ready');"],
                        {type: "text/javascript"});
  const url = URL.createObjectURL(blob);
  const worker = new Worker(url);
  t.add_cleanup(() => worker.terminate());
  await new Promise((resolve, reject) => {
    worker.onmessage = resolve;
    worker.onerror = e => reject(new Error(e.message));
  });
  URL.revokeObjectURL(url);
  worker.onmessage = null;
  return worker;
}

// Posts message to worker synchronously and resolves with the handler's
// result, rethrowing errors raised in the worker.
function callWorker(worker, message, transferList = []) {
  const reply = new Promise(resolve => {
    worker.addEventListener("message", e => resolve(e.data), {once: true});
  });
  worker.postMessage(message, transferList);
  return reply.then(({result, error}) => {
    if (error !== undefined) {
      throw new Error(`Worker: ${error}`);
    }
    return result;
  });
}

async function getTrack(t, kind) {
  const stream = await navigator.mediaDevices.getUserMedia({[kind]: true});
  t.add_cleanup(() => stream.getTracks().forEach(track => track.stop()));
  return stream.getTracks()[0];
}
