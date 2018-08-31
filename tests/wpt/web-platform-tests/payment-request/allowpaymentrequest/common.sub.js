// Test should set these:
// const expectSuccess = {'iframe': bool, 'frame': bool, 'object': bool, 'embed': bool};
// const setAllowPaymentRequest = bool;
// const testCrossOrigin = bool;

const tests = {};

window.onmessage = e => {
  const result = e.data;
  const tagName = result.urlQuery;
  const t = tests[tagName];
  t.step(() => {
    if (expectSuccess[tagName]) {
      assert_equals(result.message, "Success");
      if (result.message === "Exception") {
        const [, code, name, stack] = result.details;
        assert_unreached(`Unexpected exception "${name}" (${code}) ${stack}`);
      }
    } else {
      assert_equals(result.message, "Exception");
      const detailsArray = result.details.slice(0,3);
      assert_array_equals(detailsArray, [
        true /*ex instanceof DOMException*/,
        DOMException.SECURITY_ERR /*ex.code*/,
        "SecurityError" /*ex.name*/,
      ]);
    }
    t.done();
  });
};

["iframe", "frame", "object", "embed"].forEach((tagName, i) => {
  tests[tagName] = async_test(t => {
    const elm = document.createElement(tagName);
    if (setAllowPaymentRequest) {
      elm.setAttribute("allowpaymentrequest", "");
    }
    const path = location.pathname.substring(
      0,
      location.pathname.lastIndexOf("/") + 1
    );
    const url =
      (testCrossOrigin ? "https://{{domains[www1]}}:{{ports[https][0]}}" : "") +
      path +
      "echo-PaymentRequest.html?" +
      tagName;
    if (tagName === "object") {
      elm.data = url;
    } else {
      elm.src = url;
    }
    elm.onload = t.step_func(() => {
      window[i].postMessage(
        "What is the result of new PaymentRequest(...)?",
        "*"
      );
    });
    elm.onerror = t.unreached_func("elm.onerror");
    document.body.appendChild(elm);
  }, tagName);
});
