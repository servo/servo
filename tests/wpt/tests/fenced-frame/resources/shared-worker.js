// This is loaded as a SharedWorker in a WPT. When postMessaged to, forwards
// that message to all registered ports through a postMessage call.
const ports = [];

onconnect = function (event) {
  const port = event.ports[0];
  ports.push(port);

  port.onmessage = async function(e) {
    ports.forEach(curPort => {
      curPort.postMessage(e.data);
    });
  }
}
