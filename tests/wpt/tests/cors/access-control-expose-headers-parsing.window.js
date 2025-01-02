promise_test(() => fetch("resources/access-control-expose-headers.json").then(res => res.json()).then(runTests), "Loading JSON…");

function runTests(allTestData) {
  allTestData.forEach(testData => {
    const encodedInput = encodeURIComponent(testData.input);
    promise_test(() => {
      const relativeURL = "resources/expose-headers.py?expose=" + encodedInput,
            url = new URL(relativeURL, location.href).href.replace("://", "://élève.");
      return fetch(url).then(res => {
        assert_equals(res.headers.get("content-language"), "mkay");
        assert_equals(res.headers.get("bb-8"), (testData.exposed ? "hey" : null));
      });
    }, "Parsing: " + encodedInput);
  })
}
