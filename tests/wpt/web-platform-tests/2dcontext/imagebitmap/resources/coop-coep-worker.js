onconnect = e => {
  const port = e.source;
  port.onmessageerror = e => {
    port.postMessage("Got failure as expected.");
  }
  port.onmessage = e => {
    port.postMessage("Got message, expected failure.");
  }
}
