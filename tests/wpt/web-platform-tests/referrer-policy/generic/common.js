// NOTE: This method only strips the fragment and is not in accordance to the
// recommended draft specification:
// https://w3c.github.io/webappsec/specs/referrer-policy/#null
// TODO(kristijanburnik): Implement this helper as defined by spec once added
// scenarios for URLs containing username/password/etc.
function stripUrlForUseAsReferrer(url) {
  return url.replace(/#.*$/, "");
}

function parseUrlQueryString(queryString) {
  var queries = queryString.replace(/^\?/, "").split("&");
  var params = {};

  for (var i in queries) {
    var kvp = queries[i].split("=");
    params[kvp[0]] = kvp[1];
  }

  return params;
};

function appendIframeToBody(url, attributes) {
  var iframe = document.createElement("iframe");
  iframe.src = url;
  // Extend element with attributes. (E.g. "referrerPolicy" or "rel")
  if (attributes) {
    for (var attr in attributes) {
      iframe[attr] = attributes[attr];
    }
  }
  document.body.appendChild(iframe);

  return iframe;
}

function loadImageInWindow(src, callback, attributes, w) {
  var image = new w.Image();
  image.crossOrigin = "Anonymous";
  image.onload = function() {
    callback(image);
  }

  // Extend element with attributes. (E.g. "referrerPolicy" or "rel")
  if (attributes) {
    for (var attr in attributes) {
      image[attr] = attributes[attr];
    }
  }

  image.src = src;
  w.document.body.appendChild(image)
}

function extractImageData(img) {
    var canvas = document.createElement("canvas");
    var context = canvas.getContext('2d');
    context.drawImage(img, 0, 0);
    var imgData = context.getImageData(0, 0, img.clientWidth, img.clientHeight);
    return imgData.data;
}

function decodeImageData(rgba) {
  var rgb = new Uint8ClampedArray(rgba.length);

  // RGBA -> RGB.
  var rgb_length = 0;
  for (var i = 0; i < rgba.length; ++i) {
    // Skip alpha component.
    if (i % 4 == 3)
      continue;

    // Zero is the string terminator.
    if (rgba[i] == 0)
      break;

    rgb[rgb_length++] = rgba[i];
  }

  // Remove trailing nulls from data.
  rgb = rgb.subarray(0, rgb_length);
  var string_data = (new TextDecoder("ascii")).decode(rgb);

  return JSON.parse(string_data);
}

function normalizePort(targetPort) {
  var defaultPorts = [80, 443];
  var isDefaultPortForProtocol = (defaultPorts.indexOf(targetPort) >= 0);

  return (targetPort == "" || isDefaultPortForProtocol) ?
          "" : ":" + targetPort;
}

function wrapResult(url, server_data) {
  return {
    location: url,
    referrer: server_data.headers.referer,
    headers: server_data.headers
  }
}

function queryIframe(url, callback, referrer_policy) {
  var iframe = appendIframeToBody(url, referrer_policy);
  var listener = function(event) {
    if (event.source != iframe.contentWindow)
      return;

    callback(event.data, url);
    window.removeEventListener("message", listener);
  }
  window.addEventListener("message", listener);
}

function queryImage(url, callback, attributes, referrerPolicy, test) {
  // For images, we'll test:
  // - images in a `srcdoc` frame to ensure that it uses the referrer
  //   policy of its parent,
  // - images in a top-level document,
  // - and images in a `srcdoc` frame with its own referrer policy to
  //   override its parent.

  var noSrcDocPolicy = new Promise((resolve, reject) => {
    var iframeWithoutOwnPolicy = document.createElement('iframe');
    iframeWithoutOwnPolicy.srcdoc = "Hello, world.";
    iframeWithoutOwnPolicy.onload = function () {
      var nextUrl = url + "&cache_destroyer2=" + (new Date()).getTime();
      loadImageInWindow(nextUrl, function (img) {
        resolve(decodeImageData(extractImageData(img)));
      }, attributes, iframeWithoutOwnPolicy.contentWindow);
    };
    document.body.appendChild(iframeWithoutOwnPolicy);
  });

  // Give a srcdoc iframe a referrer policy different from the top-level page's policy.
  var iframePolicy = (referrerPolicy === "no-referrer") ? "unsafe-url" : "no-referrer";
  var srcDocPolicy = new Promise((resolve, reject) => {
    var iframeWithOwnPolicy = document.createElement('iframe');
    iframeWithOwnPolicy.srcdoc = "<meta name='referrer' content='" + iframePolicy + "'>Hello world.";

    iframeWithOwnPolicy.onload = function () {
      var nextUrl = url + "&cache_destroyer3=" + (new Date()).getTime();
      loadImageInWindow(nextUrl, function (img) {
        resolve(decodeImageData(extractImageData(img)));
      }, null, iframeWithOwnPolicy.contentWindow);
    };
    document.body.appendChild(iframeWithOwnPolicy);
  });

  var pagePolicy = new Promise((resolve, reject) => {
    loadImageInWindow(url, function (img) {
      resolve(decodeImageData(extractImageData(img)));
    }, attributes, window);
  });

  Promise.all([noSrcDocPolicy, srcDocPolicy, pagePolicy]).then(test.step_func(values => {
    assert_equals(values[0].headers.referer, values[2].headers.referer, "Referrer inside 'srcdoc' without its own policy should be the same as embedder's referrer.");
    assert_equals((iframePolicy === "no-referrer" ? undefined : document.location.href), values[1].headers.referer, "Referrer inside 'srcdoc' should use the iframe's policy if it has one");
    callback(wrapResult(url, values[2]), url);
  }));
}

function queryXhr(url, callback) {
  var xhr = new XMLHttpRequest();
  xhr.open('GET', url, true);
  xhr.onreadystatechange = function(e) {
    if (this.readyState == 4 && this.status == 200) {
      var server_data = JSON.parse(this.responseText);
      callback(wrapResult(url, server_data), url);
    }
  };
  xhr.send();
}

function queryWorker(url, callback) {
  var worker = new Worker(url);
  worker.onmessage = function(event) {
    var server_data = event.data;
    callback(wrapResult(url, server_data), url);
  };
}

function queryFetch(url, callback) {
  fetch(url).then(function(response) {
      response.json().then(function(server_data) {
        callback(wrapResult(url, server_data), url);
      });
    }
  );
}

function queryNavigable(element, url, callback, attributes) {
  var navigable = element
  navigable.href = url;
  navigable.target = "helper-iframe";

  var helperIframe = document.createElement("iframe")
  helperIframe.name = "helper-iframe"
  document.body.appendChild(helperIframe)

  // Extend element with attributes. (E.g. "referrer_policy" or "rel")
  if (attributes) {
    for (var attr in attributes) {
      navigable[attr] = attributes[attr];
    }
  }

  var listener = function(event) {
    if (event.source != helperIframe.contentWindow)
      return;

    callback(event.data, url);
    window.removeEventListener("message", listener);
  }
  window.addEventListener("message", listener);

  navigable.click();
}

function queryLink(url, callback, referrer_policy) {
  var a = document.createElement("a");
  a.innerHTML = "Link to subresource";
  document.body.appendChild(a);
  queryNavigable(a, url, callback, referrer_policy)
}

function queryAreaLink(url, callback, referrer_policy) {
  var area = document.createElement("area");
  // TODO(kristijanburnik): Append to map and add image.
  document.body.appendChild(area);
  queryNavigable(area, url, callback, referrer_policy)
}

function queryScript(url, callback, attributes, referrer_policy) {
  var script = document.createElement("script");
  script.src = url;
  script.referrerPolicy = referrer_policy;

  var listener = function(event) {
    var server_data = event.data;
    callback(wrapResult(url, server_data), url);
    window.removeEventListener("message", listener);
  }
  window.addEventListener("message", listener);

  document.body.appendChild(script);
}

 // SanityChecker does nothing in release mode.
function SanityChecker() {}
SanityChecker.prototype.checkScenario = function() {};
SanityChecker.prototype.checkSubresourceResult = function() {};
