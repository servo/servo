var expected = 'self location close onerror importScripts navigator addEventListener removeEventListener dispatchEvent name onconnect setTimeout clearTimeout setInterval clearInterval'.split(' ');
var log = '';
for (var i = 0; i < expected.length; ++i) {
  if (!(expected[i] in self))
    log += expected[i] + ' did not exist\n';
}
onconnect = function(e) {
  e.ports[0].postMessage(log);
};