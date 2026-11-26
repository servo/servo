onmessage = function(evt) {
  postMessage({
    className: Object.prototype.toString.call(evt),
    type: evt.type,
    bubbles: evt.bubbles,
    cancelable: evt.cancelable,
    data: evt.data,
    origin: evt.origin,
    lastEventId: evt.lastEventId,
    sourceIsNull: evt.source === null,
    portsLength: evt.ports.length,
  });
};
