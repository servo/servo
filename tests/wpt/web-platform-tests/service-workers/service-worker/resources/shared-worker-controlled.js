onconnect = function(e) {
  var port = e.ports[0];
  var xhr = new XMLHttpRequest();
  xhr.onload = function() { port.postMessage(this.responseText); };
  xhr.onerror = function(e) { port.postMessage(e); };
  xhr.open('GET', 'dummy.txt?simple', true);
  xhr.send();
};
