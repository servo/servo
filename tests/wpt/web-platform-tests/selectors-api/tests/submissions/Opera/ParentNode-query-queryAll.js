/*
 * Check that the query and queryAll methods exist on the given Node
 */
function interfaceCheckQuery(type, obj) {
  test(function() {
    var q = typeof obj.query === "function";
    assert_true(q, type + " supports query.");
  }, type + " supports query")

  test(function() {
    var qa = typeof obj.queryAll === "function";
    assert_true( qa, type + " supports queryAll.");
  }, type + " supports queryAll")
}

/*
 * Verify that the NodeList returned by queryAll is static and and that a new list is created after
 * each call. A static list should not be affected by subsequent changes to the DOM.
 */
function verifyStaticList(type, root) {
  var pre, post, preLength;

  test(function() {
    pre = root.queryAll("div");
    preLength = pre.length;

    var div = doc.createElement("div");
    (root.body || root).appendChild(div);

    assert_equals(pre.length, preLength, "The length of the NodeList should not change.")
  }, type + ": static NodeList")

  test(function() {
    post = root.queryAll("div"),
    assert_equals(post.length, preLength + 1, "The length of the new NodeList should be 1 more than the previous list.")
  }, type + ": new NodeList")
}

/*
 * Verify handling of special values for the selector parameter, including stringification of
 * null and undefined, and the handling of the empty string.
 */
function runSpecialSelectorTests(type, root) {
  test(function() { // 1
    assert_equals(root.queryAll(null).length, 1, "This should query one element with the tag name 'NULL'.");
  }, type + ".queryAll null")

  test(function() { // 2
    assert_equals(root.queryAll(undefined).length, 1, "This should query one elements with the tag name 'UNDEFINED'.");
  }, type + ".queryAll undefined")

  test(function() { // 3
    assert_throws(TypeError(), function() {
      root.queryAll();
    }, "This should throw a TypeError.")
  }, type + ".queryAll no parameter")

  test(function() { // 4
    var elm = root.query(null)
    assert_not_equals(elm, null, "This should query an element.");
    assert_equals(elm.tagName.toUpperCase(), "NULL", "The tag name should be 'NULL'.")
  }, type + ".query null")

  test(function() { // 5
    var elm = root.query(undefined)
    assert_not_equals(elm, undefined, "This should query an element.");
    assert_equals(elm.tagName.toUpperCase(), "UNDEFINED", "The tag name should be 'UNDEFINED'.")
  }, type + ".query undefined")

  test(function() { // 6
    assert_throws(TypeError(), function() {
      root.query();
    }, "This should throw a TypeError.")
  }, type + ".query no parameter.")

  test(function() { // 7
    result = root.queryAll("*");
    var i = 0;
    traverse(root, function(elem) {
      if (elem !== root) {
        assert_equals(elem, result[i++], "The result in index " + i + " should be in tree order.")
      }
    })
  }, type + ".queryAll tree order");
}

/*
 * Execute queries with the specified valid selectors for both query() and queryAll()
 * Only run these tests when results are expected. Don't run for syntax error tests.
 *
 * context.queryAll(selector, refNodes)
 * context.queryAll(selector)        // Only if refNodes is not specified
 * root.queryAll(selector, context)  // Only if refNodes is not specified
 * root.queryAll(selector, refNodes) // Only if context is not specified
 * root.queryAll(selector)           // Only if neither context nor refNodes is specified
 *
 * Equivalent tests will be run for .query() as well.
 */
function runValidSelectorTest(type, root, selectors, docType) {
  var nodeType = getNodeType(root);

  for (var i = 0; i < selectors.length; i++) {
    var s = selectors[i];
    var n = s["name"];
    var q = s["selector"];
    var e = s["expect"];

    var ctx = s["ctx"];
    var ref = s["ref"];

    if (!s["exclude"] || (s["exclude"].indexOf(nodeType) === -1 && s["exclude"].indexOf(docType) === -1)) {
      var foundall, found, context, refNodes, refArray;

      if (s["testType"] & TEST_FIND) {


        /*
         * If ctx and ref are specified:
         * context.queryAll(selector, refNodes)
         * context.query(selector, refNodes)
         */
        if (ctx && ref) {
          context = root.querySelector(ctx);
          refNodes = root.querySelectorAll(ref);
          refArray = Array.prototype.slice.call(refNodes, 0);

          test(function() {
            foundall = context.queryAll(q, refNodes);
            verifyNodeList(foundall, expect);
          }, type + " [Context Element].queryAll: " + n + " (with refNodes NodeList): " + q);

          test(function() {
            foundall = context.queryAll(q, refArray);
            verifyNodeList(foundall, expect);
          }, type + " [Context Element].queryAll: " + n + " (with refNodes Array): " + q);

          test(function() {
            found = context.query(q, refNodes);
            verifyElement(found, foundall, expect)
          }, type + " [Context Element].query: " + n + " (with refNodes NodeList): " + q);

          test(function() {
            found = context.query(q, refArray);
            verifyElement(found, foundall, expect)
          }, type + " [Context Element].query: " + n + " (with refNodes Array): " + q);
        }


        /*
         * If ctx is specified, ref is not:
         * context.queryAll(selector)
         * context.query(selector)
         * root.queryAll(selector, context)
         * root.query(selector, context)
         */
        if (ctx && !ref) {
          context = root.querySelector(ctx);

          test(function() {
            foundall = context.queryAll(q);
            verifyNodeList(foundall, expect);
          }, type + " [Context Element].queryAll: " + n + " (with no refNodes): " + q);

          test(function() {
            found = context.query(q);
            verifyElement(found, foundall, expect)
          }, type + " [Context Element].query: " + n + " (with no refNodes): " + q);

          test(function() {
            foundall = root.queryAll(q, context);
            verifyNodeList(foundall, expect);
          }, type + " [Root Node].queryAll: " + n + " (with refNode Element): " + q);

          test(function() {
            foundall = root.query(q, context);
            verifyElement(found, foundall, expect);
          }, type + " [Root Node].query: " + n + " (with refNode Element): " + q);
        }

        /*
         * If ref is specified, ctx is not:
         * root.queryAll(selector, refNodes)
         * root.query(selector, refNodes)
         */
        if (!ctx && ref) {
          refNodes = root.querySelectorAll(ref);
          refArray = Array.prototype.slice.call(refNodes, 0);

          test(function() {
            foundall = root.queryAll(q, refNodes);
            verifyNodeList(foundall, expect);
          }, type + " [Root Node].queryAll: " + n + " (with refNodes NodeList): " + q);

          test(function() {
            foundall = root.queryAll(q, refArray);
            verifyNodeList(foundall, expect);
          }, type + " [Root Node].queryAll: " + n + " (with refNodes Array): " + q);

          test(function() {
            found = root.query(q, refNodes);
            verifyElement(found, foundall, expect);
          }, type + " [Root Node].query: " + n + " (with refNodes NodeList): " + q);

          test(function() {
            found = root.query(q, refArray);
            verifyElement(found, foundall, expect);
          }, type + " [Root Node].query: " + n + " (with refNodes Array): " + q);
        }

        /*
         * If neither ctx nor ref is specified:
         * root.queryAll(selector)
         * root.query(selector)
         */
        if (!ctx && !ref) {
          test(function() {
            foundall = root.queryAll(q);
            verifyNodeList(foundall, expect);
          }, type + ".queryAll: " + n + " (with no refNodes): " + q);

          test(function() {
            found = root.query(q);
            verifyElement(found, foundall, expect);
          }, type + ".query: " + n + " (with no refNodes): " + q);
        }
      }
    }
  }
}

/*
 * Execute queries with the specified invalid selectors for both query() and queryAll()
 * Only run these tests when errors are expected. Don't run for valid selector tests.
 */
function runInvalidSelectorTestQuery(type, root, selectors) {
  for (var i = 0; i < selectors.length; i++) {
    var s = selectors[i];
    var n = s["name"];
    var q = s["selector"];

    test(function() {
      assert_throws("SyntaxError", function() {
        root.query(q)
      })
    }, type + ".query: " + n + ": " + q);

    test(function() {
      assert_throws("SyntaxError", function() {
        root.queryAll(q)
      })
    }, type + ".queryAll: " + n + ": " + q);
  }
}

function verifyNodeList(resultAll, expect) {
  assert_not_equals(resultAll, null, "The method should not return null.");
  assert_equals(resultAll.length, e.length, "The method should return the expected number of matches.");

  for (var i = 0; i < e.length; i++) {
    assert_not_equals(resultAll[i], null, "The item in index " + i + " should not be null.")
    assert_equals(resultAll[i].getAttribute("id"), e[i], "The item in index " + i + " should have the expected ID.");
    assert_false(resultAll[i].hasAttribute("data-clone"), "This should not be a cloned element.");
  }
}

function verifyElement(result, resultAll, expect) {
  if (expect.length > 0) {
    assert_not_equals(result, null, "The method should return a match.")
    assert_equals(found.getAttribute("id"), e[0], "The method should return the first match.");
    assert_equals(result, resultAll[0], "The result should match the first item from querySelectorAll.");
    assert_false(found.hasAttribute("data-clone"), "This should not be annotated as a cloned element.");
  } else {
    assert_equals(result, null, "The method should not match anything.");
  }
}
