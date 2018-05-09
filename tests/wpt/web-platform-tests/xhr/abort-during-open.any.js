var test = async_test("XMLHttpRequest: abort() during OPEN");
test.step(function() {
  var client = new XMLHttpRequest()
  client.open("GET", "...")
  client.onreadystatechange = function() {
    test.step(function() {
      assert_unreached()
    })
  }
  assert_equals(client.readyState, 1, "before abort()")
  assert_equals(client.status, 0)
  assert_equals(client.statusText, "")
  client.abort()
  assert_equals(client.readyState, 1, "after abort()")
  assert_equals(client.status, 0)
  assert_equals(client.statusText, "")
})
test.done()
