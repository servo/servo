var setup = async_test("Loading data…")
setup.step(function() {
  var request = new XMLHttpRequest()
  request.open("GET", "urltestdata.json")
  request.send()
  request.responseType = "json"
  request.onload = setup.step_func(function() {
    runURLTests(request.response)
    setup.done()
  })
})

function setBase(base) {
  document.getElementById("base").href = base
}

function bURL(url, base) {
  base = base || "about:blank"
  setBase(base)
  var a = document.createElement("a")
  a.setAttribute("href", url)
  return a
}

function runURLTests(urltests) {
  for(var i = 0, l = urltests.length; i < l; i++) {
    var expected = urltests[i]
    if (typeof expected === "string") continue // skip comments

    test(function() {
      var url = bURL(expected.input, expected.base)
      if(expected.failure) {
        if(url.protocol !== ':') {
          assert_unreached("Expected URL to fail parsing")
        }
        assert_equals(url.href, expected.input, "failure should set href to input")
        return
      }

      assert_equals(url.href, expected.href, "href")
      if ("origin" in expected) {
        assert_equals(url.origin, expected.origin, "origin")
      }
      assert_equals(url.protocol, expected.protocol, "protocol")
      assert_equals(url.username, expected.username, "username")
      assert_equals(url.password, expected.password, "password")
      assert_equals(url.host, expected.host, "host")
      assert_equals(url.hostname, expected.hostname, "hostname")
      assert_equals(url.port, expected.port, "port")
      assert_equals(url.pathname, expected.pathname, "pathname")
      assert_equals(url.search, expected.search, "search")
      assert_equals(url.hash, expected.hash, "hash")
    }, "Parsing: <" + expected.input + "> against <" + expected.base + ">")
  }
}
