// META: title=XMLHttpRequest: abort() during LOADING

      async_test(test => {
        var client = new XMLHttpRequest(),
            result = [],
            expected = [1, 2, 3, 4]
        client.onreadystatechange = test.step_func(function() {
          result.push(client.readyState);
          if (client.readyState === 3) {
            assert_equals(client.status, 200)
            assert_equals(client.statusText, "OK")
            assert_equals(client.responseXML, null)
            client.abort();
            assert_equals(client.readyState, 0)
            assert_equals(client.status, 0)
            assert_equals(client.statusText, "")
            assert_equals(client.responseXML, null)
            assert_equals(client.getAllResponseHeaders(), "")
          }
          if (client.readyState === 4) {
            assert_equals(client.readyState, 4)
            assert_equals(client.status, 0)
            assert_equals(client.statusText, "")
            assert_equals(client.responseXML, null)
            assert_equals(client.getAllResponseHeaders(), "")
          }
        })
        client.onloadend = test.step_func(function() {
          assert_equals(client.readyState, 4)
          assert_equals(client.status, 0)
          assert_equals(client.statusText, "")
          assert_equals(client.responseXML, null)
          assert_equals(client.getAllResponseHeaders(), "")
          test.step_timeout(function() {
            assert_array_equals(result, expected)
            test.done();
          }, 100); // wait a bit in case XHR timeout causes spurious event
        })
        client.open("GET", "resources/well-formed.xml")
        client.send(null)
      })
