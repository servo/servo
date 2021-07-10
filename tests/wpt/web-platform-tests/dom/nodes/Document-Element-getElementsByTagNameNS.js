function test_getElementsByTagNameNS(context, element) {
  test(function() {
    assert_false(context.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "html") instanceof NodeList, "NodeList")
    assert_true(context.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "html") instanceof HTMLCollection, "HTMLCollection")
    var firstCollection = context.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "html"),
        secondCollection = context.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "html")
    assert_true(firstCollection !== secondCollection || firstCollection === secondCollection,
                "Caching is allowed.")
  })

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "body"))
    this.add_cleanup(function() {element.removeChild(t)})
    var actual = context.getElementsByTagNameNS("*", "body");
    var expected = [];
    var get_elements = function(node) {
      for (var i = 0; i < node.childNodes.length; i++) {
        var child = node.childNodes[i];
        if (child.nodeType === child.ELEMENT_NODE) {
          if (child.localName == "body") {
            expected.push(child);
          }
          get_elements(child);
        }
      }
    }
    get_elements(context);
    assert_array_equals(actual, expected);
  }, "getElementsByTagNameNS('*', 'body')")

  test(function() {
    assert_array_equals(context.getElementsByTagNameNS("", "*"), []);
    var t = element.appendChild(document.createElementNS("", "body"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagNameNS("", "*"), [t]);
  }, "Empty string namespace")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "body"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagNameNS("test", "body"), [t]);
  }, "body element in test namespace, no prefix")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "test:body"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagNameNS("test", "body"), [t]);
  }, "body element in test namespace, prefix")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "BODY"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagNameNS("test", "BODY"), [t]);
    assert_array_equals(context.getElementsByTagNameNS("test", "body"), []);
  }, "BODY element in test namespace, no prefix")

  test(function() {
    var t = element.appendChild(document.createElementNS("http://www.w3.org/1999/xhtml", "abc"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "abc"), [t]);
    assert_array_equals(context.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "ABC"), []);
    assert_array_equals(context.getElementsByTagNameNS("test", "ABC"), []);
  }, "abc element in html namespace")

  test(function() {
    var t = element.appendChild(document.createElementNS("http://www.w3.org/1999/xhtml", "ABC"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "abc"), []);
    assert_array_equals(context.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "ABC"), [t]);
  }, "ABC element in html namespace")

  test(function() {
    var t = element.appendChild(document.createElementNS("http://www.w3.org/1999/xhtml", "AÇ"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "AÇ"), [t]);
    assert_array_equals(context.getElementsByTagNameNS("test", "aÇ"), []);
    assert_array_equals(context.getElementsByTagNameNS("test", "aç"), []);
  }, "AÇ, case sensitivity")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "test:BODY"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagNameNS("test", "BODY"), [t]);
    assert_array_equals(context.getElementsByTagNameNS("test", "body"), []);
  }, "BODY element in test namespace, prefix")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "test:test"))
    this.add_cleanup(function() {element.removeChild(t)})
    var actual = context.getElementsByTagNameNS("http://www.w3.org/1999/xhtml", "*");
    var expected = [];
    var get_elements = function(node) {
      for (var i = 0; i < node.childNodes.length; i++) {
        var child = node.childNodes[i];
        if (child.nodeType === child.ELEMENT_NODE) {
          if (child !== t) {
            expected.push(child);
          }
          get_elements(child);
        }
      }
    }
    get_elements(context);
    assert_array_equals(actual, expected);
  }, "getElementsByTagNameNS('http://www.w3.org/1999/xhtml', '*')")

  test(function() {
    var actual = context.getElementsByTagNameNS("*", "*");
    var expected = [];
    var get_elements = function(node) {
      for (var i = 0; i < node.childNodes.length; i++) {
        var child = node.childNodes[i];
        if (child.nodeType === child.ELEMENT_NODE) {
          expected.push(child);
          get_elements(child);
        }
      }
    }
    get_elements(context);
    assert_array_equals(actual, expected);
  }, "getElementsByTagNameNS('*', '*')")

  test(function() {
    assert_array_equals(context.getElementsByTagNameNS("**", "*"), []);
    assert_array_equals(context.getElementsByTagNameNS(null, "0"), []);
    assert_array_equals(context.getElementsByTagNameNS(null, "div"), []);
  }, "Empty lists")

  test(function() {
    var t1 = element.appendChild(document.createElementNS("test", "abc"));
    this.add_cleanup(function() {element.removeChild(t1)});

    var l = context.getElementsByTagNameNS("test", "abc");
    assert_true(l instanceof HTMLCollection);
    assert_equals(l.length, 1);

    var t2 = element.appendChild(document.createElementNS("test", "abc"));
    assert_equals(l.length, 2);

    element.removeChild(t2);
    assert_equals(l.length, 1);
  }, "getElementsByTagNameNS() should be a live collection");
}
