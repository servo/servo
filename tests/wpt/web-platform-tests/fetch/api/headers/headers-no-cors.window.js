promise_test(() => fetch("../cors/resources/not-cors-safelisted.json").then(res => res.json().then(runTests)), "Loading data…");

function runTests(testArray) {
  testArray = testArray.concat([
    ["dpr", "2"],
    ["downlink", "1"], // https://wicg.github.io/netinfo/
    ["save-data", "on"],
    ["viewport-width", "100"],
    ["width", "100"]
  ]);
  testArray.forEach(testItem => {
    const [headerName, headerValue] = testItem;
    test(() => {
      const noCorsHeaders = new Request("about:blank", { mode: "no-cors" }).headers;
      noCorsHeaders.append(headerName, headerValue);
      assert_false(noCorsHeaders.has(headerName));
    }, "\"no-cors\" Headers object cannot have " + headerName + "/" + headerValue + " as header");
  });
}
