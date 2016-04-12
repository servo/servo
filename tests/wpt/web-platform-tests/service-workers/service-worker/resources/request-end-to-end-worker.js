var port = undefined;

onmessage = function(e) {
  var message = e.data;
  if (typeof message === 'object' && 'port' in message) {
    port = message.port;
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
