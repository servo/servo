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
  WEBSOCKET: "websocket",
};

// These tests rely on some unintuitive cleverness due to WPT's test setup:
// 'Upgrade-Insecure-Requests' does not upgrade the port number, so we use URLs
// in the form `http://[domain]:[https-port]`. If the upgrade fails, the load will fail,
// as we don't serve HTTP over the secure port.
function generateURL(host, protocol, resourceType) {
  var url = new URL("http://{{host}}:{{ports[https][0]}}/upgrade-insecure-requests/support/");
  url.protocol = protocol == Protocol.INSECURE ? "http" : "https";
  url.hostname = host == Host.SAME_ORIGIN ? "{{host}}" : "{{domains[天気の良い日]}}";

  if (resourceType == ResourceType.IMAGE) {
    url.pathname += "pass.png";
  } else if (resourceType == ResourceType.FRAME) {
    url.pathname += "post-origin-to-parent.html";
  } else if (resourceType == ResourceType.WEBSOCKET) {
    url.port = {{ports[wss][0]}};
    url.protocol = protocol == Protocol.INSECURE ? "ws" : "wss";
    url.pathname = "echo";
  }
  return {
    name: protocol + "/" + host + " "  + resourceType,
    url: url.toString()
  };
}

function generateRedirect(host, protocol, target) {
  var url = "http://{{host}}:{{ports[https][0]}}/common/redirect.py?location=" + encodeURIComponent(target.url);
  url.protocol = protocol == Protocol.INSECURE ? "http" : "https";
  url.hostname = host == Host.SAME_ORIGIN ? "{{host}}" : "{{domains[天気の良い日]}}";
  return {
    name: protocol + "/" + host + " => " + target.name,
    url: url.toString()
  };
}

function assert_image_loads(test, url, height, width) {
  var i = document.createElement('img');
  i.onload = test.step_func_done(_ => {
    assert_equals(i.naturalHeight, height, "Height.");
    assert_equals(i.naturalWidth, width, "Width.");
  });
  i.onerror = test.unreached_func(url + " should load successfully.");
  i.src = url;
}

function assert_image_loads_in_srcdoc(test, url, height, width) {
  var frame = document.createElement('iframe');
  frame.srcdoc = "yay!";
  frame.onload = _ => {
    var i = frame.contentDocument.createElement('img');
    i.onload = test.step_func_done(_ => {
      assert_equals(i.naturalHeight, height, "Height.");
      assert_equals(i.naturalWidth, width, "Width.");
      frame.remove();
    });
    i.onerror = test.unreached_func(url + " should load successfully.");
    i.src = url;
  };

  document.body.appendChild(frame);
}

function assert_image_loads_in_blank(test, url, height, width) {
  var frame = document.createElement('iframe');
  frame.onload = _ => {
    var i = frame.contentDocument.createElement('img');
    i.onload = test.step_func_done(_ => {
      assert_equals(i.naturalHeight, height, "Height.");
      assert_equals(i.naturalWidth, width, "Width.");
      frame.remove();
    });
    i.onerror = test.unreached_func(url + " should load successfully.");
    i.src = url;
  };

  document.body.appendChild(frame);
}

function assert_frame_loads(test, url) {
  var i = document.createElement('iframe');

  window.addEventListener('message', test.step_func(e => {
    if (e.source == i.contentWindow) {
      i.remove();
      test.done();
    }
  }));

  i.src = url;
  document.body.appendChild(i);
}

function assert_websocket_loads(test, url) {
  var w = new WebSocket(url, "echo");
  w.onopen = test.step_func(_ => {
    w.onclose = test.step_func_done();
    w.close();
  });
  w.onclose = test.unreached_func("WebSocket should not close before open is called.");
}
