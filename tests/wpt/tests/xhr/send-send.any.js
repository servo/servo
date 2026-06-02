test(function() {
  var client = new XMLHttpRequest()
  client.open("GET", "resources/well-formed.xml")
  client.send(null)
  assert_throws_dom("InvalidStateError", function() { client.send(null) })
  client.abort()
}, "XMLHttpRequest: send() - send()");
