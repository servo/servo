onconnect = e => {
  e.ports[0].postMessage(true);
};
