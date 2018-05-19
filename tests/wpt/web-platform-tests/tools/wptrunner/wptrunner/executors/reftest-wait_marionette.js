var callback = arguments[arguments.length - 1];

function test(x) {
  if (!root.classList.contains("reftest-wait")) {
    observer.disconnect();
    callback();
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
