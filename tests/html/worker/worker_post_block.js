var prev = Date.now()
for (var i=0; true; i++) {

  if (i % 100000000 == 0) {
    var now = Date.now();
    postMessage(now - prev);
    prev = now;
  }
}
