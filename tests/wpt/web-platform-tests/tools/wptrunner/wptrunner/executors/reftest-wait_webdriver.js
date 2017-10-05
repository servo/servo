var callback = arguments[arguments.length - 1];

function test(x) {
  if (!root.classList.contains("reftest-wait")) {
    observer.disconnect();

    // As of 2017-04-05, the Chromium web browser exhibits a rendering bug
    // (https://bugs.chromium.org/p/chromium/issues/detail?id=708757) that
    // produces instability during screen capture. The following use of
    // `requestAnimationFrame` is intended as a short-term workaround, though
    // it is not guaranteed to resolve the issue.
    //
    // For further detail, see:
    // https://github.com/jugglinmike/chrome-screenshot-race/issues/1

    requestAnimationFrame(function() {
      requestAnimationFrame(function() {
        callback();
      });
    });
  }
}

var root = document.documentElement;
var observer = new MutationObserver(test);

observer.observe(root, {attributes: true});

if (document.readyState != "complete") {
    onload = test;
} else {
    test();
}
