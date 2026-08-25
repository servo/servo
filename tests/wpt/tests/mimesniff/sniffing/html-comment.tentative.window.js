promise_test(async t => {
  const response = await fetch("support/html-comment?pipe=header(Content-Type,DELETE)");
  const text = await response.text();
  assert_true(text.startsWith("<!--"));

  const popup = window.open("support/html-comment?pipe=header(Content-Type,DELETE)");
  t.add_cleanup(() => popup.close());

  await new Promise(resolve => {
    popup.onload = resolve;
  });

  assert_equals(popup.document.contentType, "text/html");
  assert_equals(popup.document.documentElement.localName, "html");
  assert_equals(popup.document.documentElement.namespaceURI, "http://www.w3.org/1999/xhtml");
}, "Content starting with <!-- without space or > is sniffed as HTML");
