function testRemove(node, parent, type) {
  test(function() {
    assert_true("remove" in node);
    assert_equals(typeof node.remove, "function");
    assert_equals(node.remove.length, 0);
  }, type + " should support remove()");
  test(function() {
    assert_equals(node.parentNode, null, "Node should not have a parent");
    assert_equals(node.remove(), undefined);
    assert_equals(node.parentNode, null, "Removed new node should not have a parent");
  }, "remove() should work if " + type + " doesn't have a parent");
  test(function() {
    assert_equals(node.parentNode, null, "Node should not have a parent");
    parent.appendChild(node);
    assert_equals(node.parentNode, parent, "Appended node should have a parent");
    assert_equals(node.remove(), undefined);
    assert_equals(node.parentNode, null, "Removed node should not have a parent");
    assert_array_equals(parent.childNodes, [], "Parent should not have children");
  }, "remove() should work if " + type + " does have a parent");
  test(function() {
    assert_equals(node.parentNode, null, "Node should not have a parent");
    var before = parent.appendChild(document.createComment("before"));
    parent.appendChild(node);
    var after = parent.appendChild(document.createComment("after"));
    assert_equals(node.parentNode, parent, "Appended node should have a parent");
    assert_equals(node.remove(), undefined);
    assert_equals(node.parentNode, null, "Removed node should not have a parent");
    assert_array_equals(parent.childNodes, [before, after], "Parent should have two children left");
  }, "remove() should work if " + type + " does have a parent and siblings");
}
