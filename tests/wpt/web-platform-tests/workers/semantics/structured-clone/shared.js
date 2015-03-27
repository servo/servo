importScripts('worker-common.js');
onconnect = function(connect_ev) {
  connect_ev.ports[0].onmessage = function(message_ev) {
    check(message_ev.data, this);
  };
};
