var port = undefined;
// Create a then-able object that is never resolved.
function createPending() {
  return { then: createPending };
}

onmessage = function(e) {
  var message = e.data;
  if (typeof message === 'object' && 'port' in message) {
    port = message.port;

    port.postMessage('received port');
    // The ServiceWorker which handles the "message" event must persist long
    // enough to handle the subsequent "fetch" event. To promote test
    // simplicity, the worker prevents its own termination indefinitely via a
    // then-able that is never resolved.
    e.waitUntil(createPending());
  }
};

onfetch = function(e) {
  var headers = {};
  var errorNameWhileAppendingHeader;
  for (var header of e.request.headers) {
    var key = header[0], value = header[1];
    headers[key] = value;
  }
  var errorNameWhileAddingHeader = '';
  try {
    e.request.headers.append('Test-Header', 'TestValue');
  } catch (e) {
    errorNameWhileAppendingHeader = e.name;
  }
  port.postMessage({
      url: e.request.url,
      mode: e.request.mode,
      method: e.request.method,
      referrer: e.request.referrer,
      headers: headers,
      headerSize: e.request.headers.size,
      errorNameWhileAppendingHeader: errorNameWhileAppendingHeader
    });
};
