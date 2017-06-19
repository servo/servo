function exposeTest(resource, desc) {
  const url = new URL("resources/" + resource, location.href).href.replace("://", "://élève.");

  promise_test(() => {
    return fetch(url).then(res => {
      assert_equals(res.headers.get("content-language"), "sure")
      assert_equals(res.headers.get("x-custom"), null);
    })
  }, "Access-Control-Expose-Headers parsing: " + desc);
}

exposeTest("access-control-expose-headers-parsing.asis", "#1");
exposeTest("access-control-expose-headers-parsing-2.asis", "#2")
