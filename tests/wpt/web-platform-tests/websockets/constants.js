//This file requires server-side substitutions and must be included as constants.js?pipe=sub

var PORT = "{{ports[ws][0]}}";
var PORT_SSL = "{{ports[wss][0]}}";
var PORT_H2 = "{{ports[h2][0]}}";

var SCHEME_DOMAIN_PORT;
if (location.search == '?wss') {
  SCHEME_DOMAIN_PORT = 'wss://{{host}}:' + PORT_SSL;
} else if (location.search == '?wpt_flags=h2') {
  SCHEME_DOMAIN_PORT = 'wss://{{host}}:' + PORT_H2;
} else {
  SCHEME_DOMAIN_PORT = 'ws://{{host}}:' + PORT;
}
