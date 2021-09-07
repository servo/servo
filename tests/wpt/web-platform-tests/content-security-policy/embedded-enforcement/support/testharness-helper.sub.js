const Host = {
  SAME_ORIGIN: "same-origin",
  CROSS_ORIGIN: "cross-origin",
};

const PolicyHeader = {
  CSP: "echo-policy.py?policy=",
  CSP_MULTIPLE: "echo-policy-multiple.py",
  REQUIRED_CSP: "echo-required-csp.py",
  ALLOW_CSP_FROM: "echo-allow-csp-from.py",
};

const IframeLoad = {
  EXPECT_BLOCK: true,
  EXPECT_LOAD: false,
};

function getOrigin() {
  var url = new URL("http://{{host}}:{{ports[http][0]}}/");
  return url.origin;
}

function getCrossOrigin() {
  var url = new URL("http://{{domains[天気の良い日]}}:{{ports[http][0]}}/");
  return url.toString();
}

function getSecureCrossOrigin() {
  // Since wptserve spins up servers on non-default port, 'self' matches
  // http://[host]:[specified-port] and https://[host]:[specified-port], but not
  // https://[host]:[https-port]. So, we use the http port for this https origin
  // in order to verify that a secure variant of a non-secure URL matches 'self'.
  var url = new URL("https://{{domains[天気の良い日]}}:{{ports[http][0]}}");
  return url.toString();
}

function generateURL(host, path, include_second_level_iframe, second_level_iframe_csp) {
  var url = new URL("http://{{host}}:{{ports[http][0]}}/content-security-policy/embedded-enforcement/support/");
  url.hostname = host == Host.SAME_ORIGIN ? "{{host}}" : "{{domains[天気の良い日]}}";
  url.pathname += path;
  if (include_second_level_iframe) {
    url.searchParams.append("include_second_level_iframe", "");
    if (second_level_iframe_csp)
      url.searchParams.append("second_level_iframe_csp", second_level_iframe_csp);
  }

  return url;
}

function generateURLString(host, path) {
  return generateURL(host, path, false, "").toString();
}

function generateURLStringWithSecondIframeParams(host, path, second_level_iframe_csp) {
  return generateURL(host, path, true, second_level_iframe_csp).toString();
}

function generateRedirect(host, target) {
  var url = new URL("http://{{host}}:{{ports[http][0]}}/common/redirect.py?location=" +
   encodeURIComponent(target));
  url.hostname = host == Host.SAME_ORIGIN ? "{{host}}" : "{{domains[天気の良い日]}}";

  return url.toString();
}

function generateUrlWithPolicies(host, policy) {
  var url = generateURL(host, PolicyHeader.CSP_MULTIPLE);
  if (policy != null)
    url.searchParams.append("policy", policy);
  return url;
}

function generateUrlWithAllowCSPFrom(host, allowCspFrom) {
  var url = generateURL(host, PolicyHeader.ALLOW_CSP_FROM);
  if (allowCspFrom != null)
    url.searchParams.append("allow_csp_from", allowCspFrom);
  return url;
}

function assert_required_csp(t, url, csp, expected) {
  var i = document.createElement('iframe');
  if(csp)
    i.csp = csp;
  i.src = url;

  window.addEventListener('message', t.step_func(e => {
    if (e.source != i.contentWindow || !('required_csp' in e.data))
      return;

    if (expected.indexOf(e.data['required_csp']) == -1)
      assert_unreached('Child iframes have unexpected csp:"' + e.data['required_csp'] + '"');

    expected.splice(expected.indexOf(e.data['required_csp']), 1);

    if (e.data['test_header_injection'] != null)
      assert_unreached('HTTP header injection was successful');

    if (expected.length == 0)
      t.done();
  }));

  document.body.appendChild(i);
}

function assert_iframe_with_csp(t, url, csp, shouldBlock, urlId, blockedURI) {
  var i = document.createElement('iframe');
  url.searchParams.append("id", urlId);
  i.src = url.toString();
  if (csp != null)
    i.csp = csp;

  var loaded = {};
  window.addEventListener("message", function (e) {
    if (e.source != i.contentWindow)
        return;
    if (e.data["loaded"])
        loaded[e.data["id"]] = true;
  });

  if (shouldBlock) {
    // Assert iframe does not load and is inaccessible.
    window.onmessage = t.step_func(function(e) {
      if (e.source != i.contentWindow)
          return;
      assert_unreached('No message should be sent from the frame.');
    });
    i.onload = t.step_func(function () {
      // Delay the check until after the postMessage has a chance to execute.
      setTimeout(t.step_func_done(function () {
        assert_equals(loaded[urlId], undefined);
      }), 500);
      assert_throws_dom("SecurityError", () => {
        var x = i.contentWindow.location.href;
      });
    });
  } else if (blockedURI) {
    // Assert iframe loads with an expected violation.
    window.addEventListener('message', t.step_func(e => {
      if (e.source != i.contentWindow)
        return;
      if (!e.data.securitypolicyviolation)
        return;
      assert_equals(e.data["blockedURI"], blockedURI);
      t.done();
    }));
  } else {
    // Assert iframe loads.  Wait for the load event, the postMessage from the
    // script and the img load event.
    let postMessage_received = false;
    let img_loaded = false;
    window.addEventListener('message', t.step_func(e => {
      if (e.source != i.contentWindow)
        return;
      if (e.data.loaded) {
        assert_true(loaded[urlId]);
        postMessage_received = true;
      } else if (e.data === "img.loaded")
        img_loaded = true;

      if (i.onloadReceived && postMessage_received && img_loaded)
        t.done();
    }));
    i.onload = t.step_func(function () {
      if (loaded[urlId])
        t.done();
      i.onloadReceived = true;
    });
  }
  document.body.appendChild(i);
}
