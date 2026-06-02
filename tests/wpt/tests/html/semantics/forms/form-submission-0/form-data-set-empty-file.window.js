test(t => {
  const form = document.body.appendChild(document.createElement("form")),
        input = form.appendChild(document.createElement("input"));
  input.type = "file";
  input.name = "hi";
  t.add_cleanup(() => {
    document.body.removeChild(form);
  });
  const fd = new FormData(form),
        value = fd.get(input.name);
  assert_true(value instanceof File, "value is a File");
  assert_equals(value.name, "", "name");
  assert_equals(value.type, "application/octet-stream", "type");
  assert_equals(value.size, 0, "expected value to be an empty File");
}, "Empty <input type=file> is still added to the form's entry list");

async_test((t) => {
  const form = document.body.appendChild(document.createElement("form")),
        input = form.appendChild(document.createElement("input")),
        target = document.createElement("iframe");
  target.name = "target1";
  document.body.appendChild(target);
  form.method = "POST";
  form.action = "/fetch/api/resources/echo-content.py";
  form.enctype = "application/x-www-form-urlencoded";
  form.target = target.name;
  input.type = "file";
  input.name = "hi";
  t.add_cleanup(() => {
    document.body.removeChild(form);
    document.body.removeChild(target);
  });

  target.addEventListener("load", t.step_func_done(() => {
    assert_equals(target.contentDocument.body.textContent, "hi=");
  }));
  form.submit();
}, "Empty <input type=file> shows up in the urlencoded serialization");

async_test((t) => {
  const form = document.body.appendChild(document.createElement("form")),
        input = form.appendChild(document.createElement("input")),
        target = document.createElement("iframe");
  target.name = "target2";
  document.body.appendChild(target);
  form.method = "POST";
  form.action = "/fetch/api/resources/echo-content.py";
  form.enctype = "multipart/form-data";
  form.target = target.name;
  input.type = "file";
  input.name = "hi";
  t.add_cleanup(() => {
    document.body.removeChild(form);
    document.body.removeChild(target);
  });

  target.addEventListener("load", t.step_func_done(() => {
    // We use \n rather than \r\n because newlines get normalized as a result
    // of HTML parsing.
    const found = target.contentDocument.body.textContent;
    const boundary = found.split("\n")[0];
    const expected = [
      boundary,
      'Content-Disposition: form-data; name="hi"; filename=""',
      "Content-Type: application/octet-stream",
      "",
      "",
      boundary + "--",
      "",
    ].join("\n");
    assert_equals(found, expected);
  }));
  form.submit();
}, "Empty <input type=file> shows up in the multipart/form-data serialization");

async_test((t) => {
  const form = document.body.appendChild(document.createElement("form")),
        input = form.appendChild(document.createElement("input")),
        target = document.createElement("iframe");
  target.name = "target3";
  document.body.appendChild(target);
  form.method = "POST";
  form.action = "/fetch/api/resources/echo-content.py";
  form.enctype = "text/plain";
  form.target = target.name;
  input.type = "file";
  input.name = "hi";
  t.add_cleanup(() => {
    document.body.removeChild(form);
    document.body.removeChild(target);
  });

  target.addEventListener("load", t.step_func_done(() => {
    // The actual result is "hi=\r\n"; the newline gets normalized as a side
    // effect of the HTML parsing.
    assert_equals(target.contentDocument.body.textContent, "hi=\n");
  }));
  form.submit();
}, "Empty <input type=file> shows up in the text/plain serialization");
