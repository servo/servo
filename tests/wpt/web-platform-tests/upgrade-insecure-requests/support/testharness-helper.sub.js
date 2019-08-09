const Host = {
  SAME_ORIGIN: "same-origin",
  CROSS_ORIGIN: "cross-origin",
};

const Protocol = {
  INSECURE: "insecure",
  SECURE: "secure",
};

const ResourceType = {
  IMAGE: "image",
  FRAME: "frame",
  WORKER: "worker",
  SHARED_WORKER: "shared-worker",
  WORKLET: "worklet",
  WEBSOCKET: "websocket",
  FETCH: "fetch",
};

// These tests rely on some unintuitive cleverness due to WPT's test setup:
// 'Upgrade-Insecure-Requests' does not upgrade the port number, so we use URLs
// in the form `http://[domain]:[https-port]`. If the upgrade fails, the load will fail,
// as we don't serve HTTP over the secure port.
function generateURL(host, protocol, resourceType) {
  var url = new URL("http://{{host}}:{{ports[https][0]}}/common/security-features/subresource/");
  url.protocol = protocol == Protocol.INSECURE ? "http" : "https";
  url.hostname = host == Host.SAME_ORIGIN ? "{{host}}" : "{{domains[天気の良い日]}}";

  if (resourceType == ResourceType.IMAGE) {
    url.pathname += "image.py";
  } else if (resourceType == ResourceType.FRAME) {
    url.pathname += "document.py";
  } else if (resourceType == ResourceType.WEBSOCKET) {
    url.port = {{ports[wss][0]}};
    url.protocol = protocol == Protocol.INSECURE ? "ws" : "wss";
    url.pathname = "echo";
  } else if (resourceType == ResourceType.WORKER) {
    url.pathname += "worker.py";
  } else if (resourceType == ResourceType.SHARED_WORKER) {
    url.pathname += "shared-worker.py";
  } else if (resourceType == ResourceType.WORKLET) {
    url.pathname += "worker.py";
  } else if (resourceType == ResourceType.FETCH) {
    url.pathname += "xhr.py";
  }
  return {
    name: protocol + "/" + host + " "  + resourceType,
    url: url.toString()
  };
}

function generateRedirect(host, protocol, finalTest) {
  var url = new URL("http://{{host}}:{{ports[https][0]}}/upgrade-insecure-requests/support/redirect-cors.py?location=" + encodeURIComponent(finalTest.url));
  url.protocol = protocol == Protocol.INSECURE ? "http" : "https";
  url.hostname = host == Host.SAME_ORIGIN ? "{{host}}" : "{{domains[天気の良い日]}}";
  return {
    name: protocol + "/" + host + " => " + finalTest.name,
    url: url.toString()
  };
}

function generateDataImport(finalTest) {
  return {
    name: "data: =(import)=> " + finalTest.name,
    url: workerUrlThatImports(finalTest.url)
  };
}

function generateTests(target, sameOriginOnly) {
  var tests = [];

  tests.push(generateURL(Host.SAME_ORIGIN, Protocol.SECURE, target));
  tests.push(generateURL(Host.SAME_ORIGIN, Protocol.INSECURE, target));
  if (!sameOriginOnly) {
    tests.push(generateURL(Host.CROSS_ORIGIN, Protocol.SECURE, target));
    tests.push(generateURL(Host.CROSS_ORIGIN, Protocol.INSECURE, target));
  }

  return tests;
}

function generateRedirectTests(target, sameOriginOnly) {
  const finalTests = generateTests(target, sameOriginOnly);
  var tests = [];

  for (const finalTest of finalTests) {
    tests.push(generateRedirect(Host.SAME_ORIGIN, Protocol.SECURE, finalTest));
    tests.push(generateRedirect(Host.SAME_ORIGIN, Protocol.INSECURE, finalTest));
    if (!sameOriginOnly) {
      tests.push(generateRedirect(Host.CROSS_ORIGIN, Protocol.SECURE, finalTest));
      tests.push(generateRedirect(Host.CROSS_ORIGIN, Protocol.INSECURE, finalTest));
    }
  }
  return tests;
}

function generateModuleImportTests(target, sameOriginOnly) {
  // |sameOriginOnly| is ignored as the top-level URL (generateDataImport())
  // is always same-origin (as it is data: URL) and import()ed URLs (URLs in
  // finalTests) can be cross-origin.

  var finalTests = generateTests(target, false);
  finalTests = finalTests.concat(generateRedirectTests(target, false));

  var tests = [];
  for (const finalTest of finalTests) {
    tests.push(generateDataImport(finalTest));
  }
  return tests;
}

function assert_image_loads(test, url) {
  var i = document.createElement('img');
  i.onload = test.step_func_done(_ => {
    assert_greater_than(i.naturalHeight, 0, "Height.");
    assert_greater_than(i.naturalWidth, 0, "Width.");
  });
  i.onerror = test.unreached_func(url + " should load successfully.");
  i.src = url;
}

function assert_image_loads_in_srcdoc(test, url) {
  var frame = document.createElement('iframe');
  frame.srcdoc = "yay!";
  frame.onload = _ => {
    var i = frame.contentDocument.createElement('img');
    i.onload = test.step_func_done(_ => {
      assert_greater_than(i.naturalHeight, 0, "Height.");
      assert_greater_than(i.naturalWidth, 0, "Width.");
      frame.remove();
    });
    i.onerror = test.unreached_func(url + " should load successfully.");
    i.src = url;
  };

  document.body.appendChild(frame);
}

function assert_image_loads_in_blank(test, url) {
  var frame = document.createElement('iframe');
  frame.onload = _ => {
    var i = frame.contentDocument.createElement('img');
    i.onload = test.step_func_done(_ => {
      assert_greater_than(i.naturalHeight, 0, "Height.");
      assert_greater_than(i.naturalWidth, 0, "Width.");
      frame.remove();
    });
    i.onerror = test.unreached_func(url + " should load successfully.");
    i.src = url;
  };

  document.body.appendChild(frame);
}

function assert_websocket_loads(test, url) {
  var w = new WebSocket(url, "echo");
  w.onopen = test.step_func(_ => {
    w.onclose = test.step_func_done();
    w.close();
  });
  w.onclose = test.unreached_func("WebSocket should not close before open is called.");
}

const testMap = {
  "image": test => {
    async_test(t => assert_image_loads(t, test.url), test.name);
    async_test(t => assert_image_loads_in_srcdoc(t, test.url), test.name + " in <iframe srcdoc>");
    async_test(t => assert_image_loads_in_blank(t, test.url), test.name + " in <iframe>");
  },
  "iframe":
    test => promise_test(t => requestViaIframe(test.url), test.name),

  "worker":
    test => promise_test(
        () => requestViaDedicatedWorker(test.url, {}),
        test.name),

  "module-worker":
    test => promise_test(
        () => requestViaDedicatedWorker(test.url, {type: "module"}),
        test.name),

  "worker-subresource-fetch":
    test => promise_test(
        () => requestViaDedicatedWorker(dedicatedWorkerUrlThatFetches(test.url),
                                        {}),
        test.name),

  "shared-worker":
    test => promise_test(
        () => requestViaSharedWorker(test.url), test.name),

  "audio-worklet":
    test => promise_test(
        () => requestViaWorklet('audio', test.url), test.name),
  "animation-worklet":
    test => promise_test(
        () => requestViaWorklet('animation', test.url), test.name),
  "layout-worklet":
    test => promise_test(
        () => requestViaWorklet('layout', test.url), test.name),
  "paint-worklet":
    test => promise_test(
        () => requestViaWorklet('paint', test.url), test.name),
};
