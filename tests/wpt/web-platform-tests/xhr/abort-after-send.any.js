// META: title=XMLHttpRequest: abort() after send()
// META: script=resources/xmlhttprequest-event-order.js

      var test = async_test()
      test.step(function() {
        var client = new XMLHttpRequest(),
            control_flag = false;
        prepare_xhr_for_event_order_test(client);
        client.addEventListener("readystatechange", test.step_func(function() {
          if(client.readyState == 4) {
            control_flag = true
            if (self.GLOBAL.isWindow()) {
              assert_equals(client.responseXML, null)
            }
            assert_equals(client.responseText, "")
            assert_equals(client.status, 0)
            assert_equals(client.statusText, "")
            assert_equals(client.getAllResponseHeaders(), "")
            assert_equals(client.getResponseHeader('Content-Type'), null)
          }
        }))
        client.open("GET", "resources/well-formed.xml", true)
        client.send(null)
        client.abort()
        assert_true(control_flag)
        assert_equals(client.readyState, 0)
        assert_xhr_event_order_matches([1, "loadstart(0,0,false)", 4, "abort(0,0,false)", "loadend(0,0,false)"])
        test.done()
      })
