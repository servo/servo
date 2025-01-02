function test_create(method, iface, nodeType, nodeName) {
  ["\u000b", "a -- b", "a-", "-b", null, undefined].forEach(function(value) {
    test(function() {
      var c = document[method](value);
      var expected = String(value);
      assert_true(c instanceof iface);
      assert_true(c instanceof CharacterData);
      assert_true(c instanceof Node);
      assert_equals(c.ownerDocument, document);
      assert_equals(c.data, expected, "data");
      assert_equals(c.nodeValue, expected, "nodeValue");
      assert_equals(c.textContent, expected, "textContent");
      assert_equals(c.length, expected.length);
      assert_equals(c.nodeType, nodeType);
      assert_equals(c.nodeName, nodeName);
      assert_equals(c.hasChildNodes(), false);
      assert_equals(c.childNodes.length, 0);
      assert_equals(c.firstChild, null);
      assert_equals(c.lastChild, null);
    }, method + "(" + format_value(value) + ")");
  });
}
