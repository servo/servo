var prev = Date.now()
setInterval(function () {
  var now = Date.now();
  postMessage(now - prev);
  prev = now;
}, 500);
