function generateCrossOriginRedirectFrame() {
  var target = "http://{{domains[天気の良い日]}}:" + document.location.port + "/content-security-policy/frame-src/support/frame.html";
  var url = "/common/redirect.py?location=" + encodeURIComponent(target);
  return { url: url, target: target };
}
