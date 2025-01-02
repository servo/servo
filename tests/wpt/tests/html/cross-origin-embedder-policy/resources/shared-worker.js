onconnect = (event) => {
  const port = event.ports[0];
  port.onmessage = (event) => {
    eval(event.data);
  };
  port.postMessage('ready');
};
