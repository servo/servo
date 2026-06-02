test(t => {
  const style = document.body.appendChild(document.createElement("style"));
  const sheet = style.sheet;
  t.add_cleanup(() => style.remove());
  assert_not_equals(sheet, null);
  style.appendChild(new Comment());
  assert_not_equals(sheet, style.sheet);
}, "Mutating the style element: inserting a Comment node");

test(t => {
  const style = document.body.appendChild(document.createElement("style"));
  t.add_cleanup(() => style.remove());
  const comment = style.appendChild(new Comment());
  const sheet = style.sheet;
  comment.appendData("x");
  assert_not_equals(sheet, style.sheet);
}, "Mutating the style element: mutating a Comment node");

test(t => {
  const style = document.body.appendChild(document.createElement("style"));
  t.add_cleanup(() => style.remove());
  const text1 = style.appendChild(new Text("1"));
  const text2 = style.appendChild(new Text("2"));
  assert_equals(style.textContent, "12");
  assert_equals(style.childNodes.length, 2);
  const sheet = style.sheet;
  style.normalize();
  assert_equals(style.childNodes.length, 1);
  assert_not_equals(sheet, style.sheet);
}, "Mutating the style element: using normalize()");

test(t => {
  const style = document.body.appendChild(document.createElement("style"));
  t.add_cleanup(() => style.remove());
  const comment = style.appendChild(new Comment());
  const sheet = style.sheet;
  comment.remove();
  assert_not_equals(sheet, style.sheet);
}, "Mutating the style element: removing a Comment node");

test(t => {
  const style = document.body.appendChild(document.createElement("style"));
  const sheet = style.sheet;
  t.add_cleanup(() => style.remove());
  assert_not_equals(sheet, null);
  style.appendChild(new DocumentFragment());
  assert_equals(sheet, style.sheet);
}, "Mutating the style element: inserting an empty DocumentFragment node");
