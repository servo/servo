// Step 2/4 (workers/same-site-cookies/{})
self.onconnect = (e) => {
    e.ports[0].postMessage("DidStart");
    self.close();
}
