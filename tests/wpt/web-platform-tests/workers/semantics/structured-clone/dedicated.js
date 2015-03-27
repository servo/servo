importScripts('worker-common.js');
onmessage = function(ev) {
  check(ev.data, self);
};
