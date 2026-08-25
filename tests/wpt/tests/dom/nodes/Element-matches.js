/*
 * Check that the matches() method exists on the given Node
 */
function interfaceCheckMatches(method, type, obj) {
  if (obj.nodeType === obj.ELEMENT_NODE) {
    test(function() {
      assert_idl_attribute(obj, method, type + " supports " + method);
    }, type + " supports " + method)
  } else {
    test(function() {
      assert_false(method in obj, type + " supports " + method);
    }, type + " should not support " + method)
  }
}

function runSpecialMatchesTests(method, type, element) {
  test(function() { // 1
    if (element.tagName.toLowerCase() === "null") {
      assert_true(element[method](null), "An element with the tag name '" + element.tagName.toLowerCase() + "' should match.");
    } else {
      assert_false(element[method](null), "An element with the tag name '" + element.tagName.toLowerCase() + "' should not match.");
    }
  }, type + "." + method + "(null)")

  test(function() { // 2
    if (element.tagName.toLowerCase() === "undefined") {
      assert_true(element[method](undefined), "An element with the tag name '" + element.tagName.toLowerCase() + "' should match.");
    } else {
      assert_false(element[method](undefined), "An element with the tag name '" + element.tagName.toLowerCase() + "' should not match.");
    }
  }, type + "." + method + "(undefined)")

  test(function() { // 3
    assert_throws_js(element.ownerDocument.defaultView.TypeError, function() {
      element[method]();
    }, "This should throw a TypeError.")
  }, type + "." + method + " no parameter")
}

/*
 * Execute queries with the specified invalid selectors for matches()
 * Only run these tests when errors are expected. Don't run for valid selector tests.
 */
function runInvalidSelectorTestMatches(method, type, root, selectors) {
  if (root.nodeType === root.ELEMENT_NODE) {
    for (var i = 0; i < selectors.length; i++) {
      var s = selectors[i];
      var n = s["name"];
      var q = s["selector"];

      test(function() {
        assert_throws_dom(
          "SyntaxError",
          root.ownerDocument.defaultView.DOMException,
          function() {
            root[method](q)
          }
        );
      }, type + "." + method + ": " + n + ": " + q);
    }
  }
}

function runMatchesTest(method, type, root, selectors, docType) {
  var nodeType = getNodeType(root);

  for (var i = 0; i < selectors.length; i++) {
    var s = selectors[i];
    var n = s["name"];
    var q = s["selector"];
    var e = s["expect"];
    var u = s["unexpected"];

    var ctx = s["ctx"];
    var ref = s["ref"];

    if ((!s["exclude"] || (s["exclude"].indexOf(nodeType) === -1 && s["exclude"].indexOf(docType) === -1))
     && (s["testType"] & TEST_MATCH) ) {

      if (ctx && !ref) {
        test(function() {
          var j, element, refNode;
          for (j = 0; j < e.length; j++) {
            element = root.querySelector("#" + e[j]);
            refNode = root.querySelector(ctx);
            assert_true(element[method](q, refNode), "The element #" + e[j] + " should match the selector.")
          }

          if (u) {
            for (j = 0; j < u.length; j++) {
              element = root.querySelector("#" + u[j]);
              refNode = root.querySelector(ctx);
              assert_false(element[method](q, refNode), "The element #" + u[j] + " should not match the selector.")
            }
          }
        }, type + " Element." + method + ": " + n + " (with refNode Element): " + q);
      }

      if (ref) {
        test(function() {
          var j, element, refNodes;
          for (j = 0; j < e.length; j++) {
            element = root.querySelector("#" + e[j]);
            refNodes = root.querySelectorAll(ref);
            assert_true(element[method](q, refNodes), "The element #" + e[j] + " should match the selector.")
          }

          if (u) {
            for (j = 0; j < u.length; j++) {
              element = root.querySelector("#" + u[j]);
              refNodes = root.querySelectorAll(ref);
              assert_false(element[method](q, refNodes), "The element #" + u[j] + " should not match the selector.")
            }
          }
        }, type + " Element." + method + ": " + n + " (with refNodes NodeList): " + q);
      }

      if (!ctx && !ref) {
        test(function() {
          for (var j = 0; j < e.length; j++) {
            var element = root.querySelector("#" + e[j]);
            assert_true(element[method](q), "The element #" + e[j] + " should match the selector.")
          }

          if (u) {
            for (j = 0; j < u.length; j++) {
              element = root.querySelector("#" + u[j]);
              assert_false(element[method](q), "The element #" + u[j] + " should not match the selector.")
            }
          }
        }, type + " Element." + method + ": " + n + " (with no refNodes): " + q);
      }
    }
  }
}
