function xfo_simple_tests({ headerValue, headerValue2, cspValue, sameOriginAllowed, crossOriginAllowed }) {
  simpleXFOTestsInner({
    urlPrefix: "",
    allowed: sameOriginAllowed,
    headerValue,
    headerValue2,
    cspValue,
    sameOrCross: "same-origin"
  });

  simpleXFOTestsInner({
    urlPrefix: "http://{{domains[www]}}:{{ports[http][0]}}",
    allowed: crossOriginAllowed,
    headerValue,
    headerValue2,
    cspValue,
    sameOrCross: "cross-origin"
  });
}

function simpleXFOTestsInner({ urlPrefix, allowed, headerValue, headerValue2, cspValue, sameOrCross }) {
  const value2QueryString = headerValue2 !== undefined ? `&value2=${headerValue2}` : ``;
  const cspQueryString = cspValue !== undefined ? `&csp_value=${cspValue}` : ``;

  const valueMessageString = headerValue === "" ? "(the empty string)" : headerValue;
  const value2MessageString = headerValue2 === "" ? "(the empty string)" : headerValue2;
  const value2MaybeMessageString = headerValue2 !== undefined ? `;${headerValue2}` : ``;
  const cspMessageString = cspValue !== undefined ? ` with CSP ${cspValue}` : ``;

  // This will test the multi-header variant, if headerValue2 is not undefined.
  xfo_test({
    url: `${urlPrefix}/x-frame-options/support/xfo.py?value=${headerValue}${value2QueryString}${cspQueryString}`,
    check: allowed ? "loaded message" : "no message",
    message: `\`${valueMessageString}${value2MaybeMessageString}\` ${allowed ? "allows" : "blocks"} ${sameOrCross} framing${cspMessageString}`
  });

  if (headerValue2 !== undefined && headerValue2 !== headerValue) {
    // Reversed variant
    xfo_test({
      url: `${urlPrefix}/x-frame-options/support/xfo.py?value=${headerValue2}&value2=${headerValue}${cspQueryString}`,
      check: allowed ? "loaded message" : "no message",
      message: `\`${value2MessageString};${valueMessageString}\` ${allowed ? "allows" : "blocks"} ${sameOrCross} framing${cspMessageString}`
    });

    // Comma variant
    xfo_test({
      url: `${urlPrefix}/x-frame-options/support/xfo.py?value=${headerValue},${headerValue2}${cspQueryString}`,
      check: allowed ? "loaded message" : "no message",
      message: `\`${valueMessageString},${value2MessageString}\` ${allowed ? "allows" : "blocks"} ${sameOrCross} framing${cspMessageString}`
    });

    // Comma + reversed variant
    xfo_test({
      url: `${urlPrefix}/x-frame-options/support/xfo.py?value=${headerValue2},${headerValue}${cspQueryString}`,
      check: allowed ? "loaded message" : "no message",
      message: `\`${value2MessageString},${valueMessageString}\` ${allowed ? "allows" : "blocks"} ${sameOrCross} framing${cspMessageString}`
    });
  }
}

function xfo_test({ url, check, message }) {
  async_test(t => {
    const i = document.createElement("iframe");
    i.src = url;

    switch (check) {
      case "loaded message": {
        waitForMessageFrom(i, t).then(t.step_func_done(e => {
          assert_equals(e.data, "Loaded");
        }));
        break;
      }
      case "failed message": {
        waitForMessageFrom(i, t).then(t.step_func_done(e => {
          assert_equals(e.data, "Failed");
        }));
        break;
      }
      case "no message": {
        waitForMessageFrom(i, t).then(t.unreached_func("Frame should not have sent a message."));
        i.onload = t.step_func_done(() => {
          assert_equals(i.contentDocument, null);
        });
        break;
      }
      default: {
        throw new Error("Bad test");
      }
    }

    document.body.append(i);
  }, message);
}

function waitForMessageFrom(frame, test) {
  return new Promise(resolve => {
    window.addEventListener("message", test.step_func(e => {
      if (e.source == frame.contentWindow) {
        resolve(e);
      }
    }));
  });
}
