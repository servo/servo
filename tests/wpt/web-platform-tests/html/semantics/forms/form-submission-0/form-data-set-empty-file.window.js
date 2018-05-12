promise_test(() => {
  const form = document.body.appendChild(document.createElement("form")),
        input = form.appendChild(document.createElement("input"));
  input.type = "file";
  input.name = "hi";
  const fd = new FormData(form),
        value = fd.get(input.name);
  assert_true(value instanceof File, "value is a File");
  assert_equals(value.name, "", "name");
  assert_equals(value.type, "application/octet-stream", "type");
  return new Response(value).text().then(body => {
    assert_equals(body, "", "body");
  });
}, "Empty <input type=file> is still serialized");
