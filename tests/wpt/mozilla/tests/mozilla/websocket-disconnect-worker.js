// Create a websocket and queue up a bunch of activity, then signal the parent to
// terminate this worker before the queued activity is complete.
importScripts('/websockets/websocket.sub.js');
var w = CreateWebSocket(false, true, false);
w.onopen = () => {
  postMessage("close");
  for (var i = 0; i < 1000; i++) {
    w.send('hello' + i);
  }
};
