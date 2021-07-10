// META: title=XMLHttpRequest: abort() during UNSENT

      var test = async_test()
      test.step(function() {
        var client = new XMLHttpRequest()
        client.onreadystatechange = function() {
          test.step(function() {
            assert_unreached()
          })
        }
        assert_equals(client.readyState, 0)
        assert_equals(client.status, 0)
        assert_equals(client.statusText, "")
        client.abort()
        assert_equals(client.readyState, 0)
        assert_equals(client.status, 0)
        assert_equals(client.statusText, "")
      })
      test.done()
