test(() => {
  assert_equals(document.children, document.children);
}, "ParentNode.children is [SameObject] (on Document)");
test(() => {
  var fragment = new DocumentFragment();
  assert_equals(fragment.children, fragment.children);
}, "ParentNode.children is [SameObject] (on DocumentFragment)");
test(() => {
  assert_equals(document.body.children, document.body.children);
}, "ParentNode.children is [SameObject] (on Element)");
test(() => {
  var fieldSet = document.createElement("fieldset");
  assert_equals(fieldSet.elements, fieldSet.elements);
}, "HTMLFieldSetElement.elements is [SameObject]");
test(() => {
  var table = document.createElement("table");
  assert_equals(table.rows, table.rows);
}, "HTMLTableElement.rows is [SameObject]");
test(() => {
  var tbody = document.createElement("tbody");
  assert_equals(tbody.rows, tbody.rows);
}, "HTMLTableSectionElement.rows is [SameObject]");
test(() => {
  var dataList = document.createElement("datalist");
  assert_equals(dataList.options, dataList.options);
}, "HTMLDataListElement.options is [SameObject]");
