onconnect = function(e) {
  e.ports[0].postMessage(self.name);
}
