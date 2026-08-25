// Post back the location of the worker

onconnect = function(e) {
  e.ports[0].postMessage(self.origin);
}
