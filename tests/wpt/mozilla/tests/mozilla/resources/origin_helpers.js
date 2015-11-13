var HTTP_PORT = '{{ports[http][0]}}';
var HTTPS_PORT = '{{ports[https][0]}}';
var ORIGINAL_HOST = '\'{{host}}\'';
var HTTP_ORIGIN = 'http://' + ORIGINAL_HOST + ':' + HTTP_PORT;
var HTTPS_ORIGIN = 'https://' + ORIGINAL_HOST + ':' + HTTPS_PORT;
