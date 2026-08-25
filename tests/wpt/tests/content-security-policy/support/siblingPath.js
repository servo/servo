 buildSiblingPath = function(hostPrefix, relativePath, newPort) {
  var port = newPort ? newPort : document.location.port;
  var path = document.location.pathname.substring(0, document.location.pathname.lastIndexOf('/') + 1);
  return (document.location.protocol + '//' + hostPrefix + "." + document.location.hostname + ':' + port + path + relativePath);
};