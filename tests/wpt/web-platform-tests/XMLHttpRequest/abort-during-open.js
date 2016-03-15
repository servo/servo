var test = async_test()
test.step(function() {
  var client = new XMLHttpRequest()
  client.open("GET", "...")
  client.onreadystatechange = function() {
    test.step(function() {
      assert_unreached()
    })
  }
  client.abort()
  assert_equals(client.readyState, 0)
  assert_throws("InvalidStateError", function() { client.send("test") }, "calling send() after abort()")
})
test.done()
