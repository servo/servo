// META: title=XMLHttpRequest: abort() should not reset event listeners

      var test = async_test()
      test.step(function() {
        var client = new XMLHttpRequest(),
            test = function() {}
        client.onreadystatechange = test
        client.open("GET", "resources/well-formed.xml")
        client.send(null)
        client.abort()
        assert_equals(client.onreadystatechange, test)
      })
      test.done()
