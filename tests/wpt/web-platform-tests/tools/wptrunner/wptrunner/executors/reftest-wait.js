function test(x) {
  log("classList: " + root.classList);
  if (!root.classList.contains("reftest-wait")) {
    observer.disconnect();
    marionetteScriptFinished();
  }
}

var root = document.documentElement;
var observer = new MutationObserver(test);

observer.observe(root, {attributes: true});

if (document.readyState != "complete") {
  onload = test
} else {
  test();
}
