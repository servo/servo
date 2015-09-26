// https://html.spec.whatwg.org/multipage/#dom-tbody-rows
function testRowsAttribute(localName) {
  var elem = document.createElement(localName);
  assert_equals(elem.rows.length, 0);

  elem.appendChild(document.createElement("p"));
  assert_equals(elem.rows.length, 0);

  elem.appendChild(document.createElement("tr"));
  assert_equals(elem.rows.length, 1);
}
