// https://html.spec.whatwg.org/multipage/#dom-tbody-rows
function testRowsAttribute(localName) {
  var elem = document.createElement(localName);
  assert_equals(elem.rows.length, 0);

  // Child <p> should *not* count as a row
  elem.appendChild(document.createElement("p"));
  assert_equals(elem.rows.length, 0);

  // Child <tr> should count as a row
  var childTr = document.createElement("tr");
  elem.appendChild(childTr);
  assert_equals(elem.rows.length, 1);

  // Nested table with child <tr> should *not* count as a row
  var nested = document.createElement(localName);
  nested.appendChild(document.createElement("tr"));
  var nestedTable = document.createElement("table");
  nestedTable.appendChild(nested);
  childTr.appendChild(nestedTable);
  assert_equals(elem.rows.length, 1);
}
