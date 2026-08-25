// META: title=XMLHttpRequest: abort() during DONE

      async_test(test => {
        var client = new XMLHttpRequest(),
            result = [],
            expected = [1, 4] // open() -> 1, send() -> 4
        client.onreadystatechange = test.step_func(function() {
          result.push(client.readyState)
        })
        client.open("GET", "resources/well-formed.xml", false)
        client.send(null)
        assert_equals(client.readyState, 4)
        assert_equals(client.status, 200)
        assert_equals(client.statusText, "OK")
        assert_equals(client.responseXML.documentElement.localName, "html")
        client.abort()
        assert_equals(client.readyState, 0)
        assert_equals(client.status, 0)
        assert_equals(client.statusText, "")
        assert_equals(client.responseXML, null)
        assert_equals(client.getAllResponseHeaders(), "")
        assert_array_equals(result, expected)
        test.done()
      }, document.title + " (sync)")

      async_test(test => {
        var client = new XMLHttpRequest(),
            result = [],
            expected = [1, 4] // open() -> 1, send() -> 4
        client.onreadystatechange = test.step_func(function() {
          result.push(client.readyState);
          if (client.readyState === 4) {
            assert_equals(client.readyState, 4)
            assert_equals(client.status, 200)
            assert_equals(client.statusText, "OK")
            assert_equals(client.responseXML.documentElement.localName, "html")
            client.abort();
            assert_equals(client.readyState, 0)
            assert_equals(client.status, 0)
            assert_equals(client.statusText, "")
            assert_equals(client.responseXML, null)
            assert_equals(client.getAllResponseHeaders(), "")
            test.done()
          }
        })
        client.open("GET", "resources/well-formed.xml", false)
        client.send(null)
        assert_equals(client.readyState, 0)
        assert_equals(client.status, 200)
        assert_equals(client.statusText, "OK")
        assert_equals(client.responseXML.documentElement.localName, "html")
      }, document.title + " (sync aborted in readystatechange)")

      async_test(test => {
        var client = new XMLHttpRequest(),
            result = [],
            expected = [1, 2, 3, 4]
        client.onreadystatechange = test.step_func(function() {
          result.push(client.readyState);
          if (client.readyState === 4) {
            assert_equals(client.readyState, 4)
            assert_equals(client.status, 200)
            assert_equals(client.responseXML.documentElement.localName, "html")
            client.abort();
            assert_equals(client.readyState, 0)
            assert_equals(client.status, 0)
            assert_equals(client.statusText, "")
            assert_equals(client.responseXML, null)
            assert_equals(client.getAllResponseHeaders(), "")
            test.step_timeout(function() {
              assert_array_equals(result, expected)
              test.done();
            }, 100); // wait a bit in case XHR timeout causes spurious event
          }
        })
        client.open("GET", "resources/well-formed.xml")
        client.send(null)
      }, document.title + " (async)")
