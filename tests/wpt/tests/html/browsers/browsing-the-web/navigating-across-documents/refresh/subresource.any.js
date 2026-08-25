promise_test(() => {
  return fetch("resources/refresh.py").then(response => {
    assert_equals(response.headers.get("Refresh"), "0;./refreshed.txt?\u0080\u00FF", "bytes got mapped to code points of the same value");
    assert_equals(response.url, new URL("resources/refresh.py", location).href, "Fetch API did not navigate to the Refresh URL");
  });
}, "Refresh does not affect Fetch API.");

promise_test(async t => {
  const { promise: xhrLoaded, resolve: resolveXHRLoaded, reject: rejectXHRLoaded } = Promise.withResolvers();
  const xhr = new XMLHttpRequest();
  xhr.open("GET", "resources/refresh.py");
  xhr.addEventListener("load", t.step_func(() => {
    assert_equals(xhr.getResponseHeader("Refresh"), "0;./refreshed.txt?\u0080\u00FF", "bytes got mapped to code points of the same value");
    assert_equals(xhr.responseURL, new URL("resources/refresh.py", location).href, "XMLHttpRequest did not navigate to the Refresh URL");
    resolveXHRLoaded();
  }));
  xhr.addEventListener("error", t.step_func(() => {
    assert_false(true, "XMLHttpRequest did not navigate to the Refresh URL");
    rejectXHRLoaded();
  }));
  xhr.send();
  return xhrLoaded;
}, "Refresh does not affect XMLHttpRequest.");

if (self.GLOBAL.isWindow()) {
  promise_test(async t => {
    const { promise: imgLoaded, resolve: resolveImgLoaded, reject: rejectImgLoaded } = Promise.withResolvers();
    const svgPath = "resources/refresh-with-svg.py";
    const img = document.createElement("img");
    img.src = svgPath;
    img.addEventListener("load", () => {
      assert_equals(img.src, new URL(svgPath, location).href, "Image did not navigate to the Refresh URL");
      resolveImgLoaded();
    });
    img.addEventListener("error", t.step_func(() => {
      assert_false(true, "Image did not navigate to the Refresh URL");
      rejectImgLoaded();
    }));
    document.documentElement.appendChild(img);
    return imgLoaded;
  }, "Refresh does not affect Image.");
}
