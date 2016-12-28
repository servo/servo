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
    if (typeof expected === "string" || !("origin" in expected)) continue

    test(function() {
      var url = bURL(expected.input, expected.base)
      assert_equals(url.origin, expected.origin, "origin")
    }, "Parsing origin: <" + expected.input + "> against <" + expected.base + ">")
  }
}
