var httpHostMain = '{{domains[]}}'; //name of the server that this page must accessed over port 80
var httpHostAlias = '{{domains[www]}}'; //another hostname (must be a subdomain so document.domain can be set to a higher domain) that accesses the same content, over HTTP
var httpsHostAlias = httpHostAlias; //another hostname (can be same as httpHostAlias) that accesses the same content, over HTTPS port
var httpPortAlias = {{ports[http][0]}}; //another port that accesses the same content on the current hostname, over HTTP
//XXX HTTPS
//var httpsPortAlias = ports[https][0]; //another port that accesses the same content on the httpsHostAlias, over HTTPS
var httpsPortAlias = 8443;

function crossOriginUrl(subdomain, relative_url) {
  var a = document.createElement("a");
  a.href = relative_url;
  return a.href.replace(location.href.replace("://", "://" + subdomain + "."));
}
