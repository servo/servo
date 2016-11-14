var test = async_test()
test.step(function() {
  var client = new XMLHttpRequest()
  client.open("GET", "...")
  client.onreadystatechange = function() {
    test.step(function() {
      assert_unreached()
    })
  }
  assert_equals(client.readyState, 1, "before abort()")
  client.abort()
  assert_equals(client.readyState, 1, "after abort()")
})
test.done()
