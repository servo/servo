function attr_is(attr, v, ln, ns, p, n, description) {
  description = description ? description + ": " : "";
  assert_equals(attr.value, v, description + "value")
  assert_equals(attr.nodeValue, v, description + "nodeValue")
  assert_equals(attr.textContent, v, description + "textContent")
  assert_equals(attr.localName, ln, description + "localName")
  assert_equals(attr.namespaceURI, ns, description + "namespaceURI")
  assert_equals(attr.prefix, p, description + "prefix")
  assert_equals(attr.name, n, description + "name")
  assert_equals(attr.nodeName, n, description + "nodeName")
  assert_equals(attr.specified, true, description + "specified")
}

function attributes_are(el, l) {
  for (var i = 0, il = l.length; i < il; i++) {
    attr_is(el.attributes[i], l[i][1], l[i][0], (l[i].length < 3) ? null : l[i][2], null, l[i][0],
      "attributes[" + i + "]")
    assert_equals(el.attributes[i].ownerElement, el)
  }
}
