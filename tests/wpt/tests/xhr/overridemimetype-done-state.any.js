// META title= XMLHttpRequest: overrideMimeType() in DONE state</title>

/**
 * Spec: <https://xhr.spec.whatwg.org/#the-overridemimetype()-method>; data-tested-assertations="/following::ol/li[1]"
 */
var test = async_test();
var client = new XMLHttpRequest();
client.onreadystatechange = test.step_func( function() {
  if (client.readyState !== 4) return;
  var text = client.responseText;
  assert_not_equals(text, "");
  assert_throws_dom("InvalidStateError", function() { client.overrideMimeType('application/xml;charset=Shift-JIS'); });
  if (GLOBAL.isWindow()) {
    assert_equals(client.responseXML, null);
  }
  assert_equals(client.responseText, text);
  test.done();
});
client.open("GET", "resources/status.py?type="+encodeURIComponent('text/plain;charset=iso-8859-1')+'&content=%3Cmsg%3E%83%65%83%58%83%67%3C%2Fmsg%3E');
client.send();
