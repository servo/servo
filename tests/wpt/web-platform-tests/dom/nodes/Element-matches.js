/*
 * Check that the matches() method exists on the given Node
 */
function interfaceCheckMatches(type, obj) {
  if (obj.nodeType === obj.ELEMENT_NODE) {
    test(function() {
      assert_idl_attribute(obj, "matches", type + " supports matches");
    }, type + " supports matches")
  }
}

function runSpecialMatchesTests(type, element) {
  test(function() { // 1
    if (element.tagName.toLowerCase() === "null") {
      assert_true(element.matches(null), "An element with the tag name '" + element.tagName.toLowerCase() + "' should match.");
    } else {
      assert_false(element.matches(null), "An element with the tag name '" + element.tagName.toLowerCase() + "' should not match.");
    }
  }, type + ".matches(null)")

  test(function() { // 2
    if (element.tagName.toLowerCase() === "undefined") {
      assert_true(element.matches(undefined), "An element with the tag name '" + element.tagName.toLowerCase() + "' should match.");
    } else {
      assert_false(element.matches(undefined), "An element with the tag name '" + element.tagName.toLowerCase() + "' should not match.");
    }
  }, type + ".matches(undefined)")

  test(function() { // 3
    assert_throws(TypeError(), function() {
      element.matches();
    }, "This should throw a TypeError.")
  }, type + ".matches no parameter")
}

/*
 * Execute queries with the specified invalid selectors for matches()
 * Only run these tests when errors are expected. Don't run for valid selector tests.
 */
function runInvalidSelectorTestMatches(type, root, selectors) {
  if (root.nodeType === root.ELEMENT_NODE) {
    for (var i = 0; i < selectors.length; i++) {
      var s = selectors[i];
      var n = s["name"];
      var q = s["selector"];

      test(function() {
        assert_throws("SyntaxError", function() {
          root.matches(q)
        })
      }, type + ".matches: " + n + ": " + q);
    }
  }
}

function runMatchesTest(type, root, selectors, docType) {
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
            assert_true(element.matches(q, refNode), "The element #" + e[j] + " should match the selector.")
          }

          if (u) {
            for (j = 0; j < u.length; j++) {
              element = root.querySelector("#" + u[j]);
              refNode = root.querySelector(ctx);
              assert_false(element.matches(q, refNode), "The element #" + u[j] + " should not match the selector.")
            }
          }
        }, type + " Element.matches: " + n + " (with refNode Element): " + q);
      }

      if (ref) {
        test(function() {
          var j, element, refNodes;
          for (j = 0; j < e.length; j++) {
            element = root.querySelector("#" + e[j]);
            refNodes = root.querySelectorAll(ref);
            assert_true(element.matches(q, refNodes), "The element #" + e[j] + " should match the selector.")
          }

          if (u) {
            for (j = 0; j < u.length; j++) {
              element = root.querySelector("#" + u[j]);
              refNodes = root.querySelectorAll(ref);
              assert_false(element.matches(q, refNodes), "The element #" + u[j] + " should not match the selector.")
            }
          }
        }, type + " Element.matches: " + n + " (with refNodes NodeList): " + q);
      }

      if (!ctx && !ref) {
        test(function() {
          for (var j = 0; j < e.length; j++) {
            var element = root.querySelector("#" + e[j]);
            assert_true(element.matches(q), "The element #" + e[j] + " should match the selector.")
          }

          if (u) {
            for (j = 0; j < u.length; j++) {
              element = root.querySelector("#" + u[j]);
              assert_false(element.matches(q), "The element #" + u[j] + " should not match the selector.")
            }
          }
        }, type + " Element.matches: " + n + " (with no refNodes): " + q);
      }
    }
  }
}
