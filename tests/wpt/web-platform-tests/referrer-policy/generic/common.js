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

function appendIframeToBody(url) {
  var iframe = document.createElement("iframe");
  iframe.src = url;
  document.body.appendChild(iframe);

  return iframe;
}

function loadImage(src, callback) {
  var image = new Image();
  image.crossOrigin = "Anonymous";
  image.onload = function() {
    callback(image);
  }
  image.src = src;
  document.body.appendChild(image)
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

function decodeImage(url, callback) {
  loadImage(url, function(img) {
    var canvas = document.createElement("canvas");
    var context = canvas.getContext('2d');
    context.drawImage(img, 0, 0);
    var imgData = context.getImageData(0, 0, img.clientWidth, img.clientHeight);
    callback(decodeImageData(imgData.data))
  });
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

function queryIframe(url, callback) {
  var iframe = appendIframeToBody(url);
  var listener = function(event) {
    if (event.source != iframe.contentWindow)
      return;

    callback(event.data, url);
    window.removeEventListener("message", listener);
  }
  window.addEventListener("message", listener);
}

function queryImage(url, callback) {
  decodeImage(url, function(server_data) {
    callback(wrapResult(url, server_data), url);
  })
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

function queryScript(url, callback) {
  var script = document.createElement("script");
  script.src = url;

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

// TODO(kristijanburnik): document.origin is supported since Chrome 41,
// other browsers still don't support it. Remove once they do.
document.origin = document.origin || (location.protocol + "//" + location.host);
