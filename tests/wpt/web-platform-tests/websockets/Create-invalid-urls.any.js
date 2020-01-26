// META: script=websocket.sub.js

var wsocket;
test(function() {
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = new WebSocket("/echo")
  });
}, "Url is /echo - should throw SYNTAX_ERR");

test(function() {
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = new WebSocket("mailto:microsoft@microsoft.com")
  });
}, "Url is a mail address - should throw SYNTAX_ERR");

test(function() {
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = new WebSocket("about:blank")
  });
}, "Url is about:blank - should throw SYNTAX_ERR");

test(function() {
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = new WebSocket("?test")
  });
}, "Url is ?test - should throw SYNTAX_ERR");

test(function() {
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = new WebSocket("#test")
  });
}, "Url is #test - should throw SYNTAX_ERR");
