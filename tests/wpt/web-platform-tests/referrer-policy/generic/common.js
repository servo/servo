// NOTE: This method only strips the fragment and is not in accordance to the
// recommended draft specification:
// https://w3c.github.io/webappsec/specs/referrer-policy/#null
// TODO(burnik): Implement this helper as defined by spec once added scenarios
// for URLs containing username/password/etc.
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
