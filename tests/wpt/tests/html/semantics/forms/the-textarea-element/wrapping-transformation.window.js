test((t) => {
  const form = document.createElement("form");
  const textarea = document.createElement("textarea");
  textarea.name = "linebreakTest";
  textarea.textContent = "a\nb\rc\r\nd\n\re";
  form.appendChild(textarea);
  document.body.appendChild(form);
  t.add_cleanup(() => {
    document.body.removeChild(form);
  });

  assert_equals(textarea.textContent, "a\nb\rc\r\nd\n\re");
  assert_equals(textarea.value, "a\nb\nc\nd\n\ne");

  const formData = new FormData(form);
  assert_equals(
    formData.get("linebreakTest"),
    "a\nb\nc\nd\n\ne",
  );
}, "Textarea wrapping transformation: Newlines should be normalized to LF.");

test((t) => {
  const form = document.createElement("form");
  const textarea = document.createElement("textarea");
  textarea.name = "wrapTest";
  textarea.cols = 10;
  textarea.wrap = "hard";
  textarea.textContent =
    "Some text that is too long for the specified character width.";
  form.appendChild(textarea);
  document.body.appendChild(form);
  t.add_cleanup(() => {
    document.body.removeChild(form);
  });

  assert_true(
    !textarea.textContent.includes("\n") &&
      !textarea.textContent.includes("\r"),
    "textContent shouldn't contain any newlines",
  );

  const formData = new FormData(form);
  const formDataValue = formData.get("wrapTest");

  assert_true(
    !formDataValue.includes("\r"),
    "The wrapping done on the value must be LF, not CRLF.",
  );
  assert_true(
    formDataValue.includes("\n"),
    "The value must be wrapped.",
  );
}, "Textarea wrapping transformation: Wrapping happens with LF newlines.");


test((t) => {
  const form = document.createElement("form");
  const textarea = document.createElement("textarea");
  textarea.name = "wrapTest";
  textarea.cols = 10;
  textarea.wrap = "hard";
  textarea.textContent = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
  form.appendChild(textarea);
  document.body.appendChild(form);
  t.add_cleanup(() => {
    document.body.removeChild(form);
  });
  const formData = new FormData(form);
  assert_equals(formData.get("wrapTest"),'ABCDEFGHIJ\nKLMNOPQRST\nUVWXYZ');
}, "Textarea hard-wrapping should honor the col count unconditionally,");

function assert_roundtrips(text, exact = false) {
  test((t) => {
    const form = document.createElement("form");
    const textarea = document.createElement("textarea");
    textarea.name = "wrapTest";
    textarea.cols = 10; // Shorter than "intermingled"
    textarea.wrap = "hard";
    form.appendChild(textarea);
    document.body.appendChild(form);
    t.add_cleanup(() => {
      document.body.removeChild(form);
    });
    textarea.value = text;
    const formDataValue = new FormData(form).get("wrapTest");
    if (exact) {
      assert_equals(formDataValue, text, "Text expected to match");
    }
    textarea.value = formDataValue;
    const newFormDataValue = new FormData(form).get("wrapTest");
    assert_equals(formDataValue, newFormDataValue, "Value should round-trip");
  }, "Textarea wrapping transformation: wrapping round-trips: " + text);
}

assert_roundtrips("Some text that is too long for the specified character width.");
assert_roundtrips("Some text that is too long for the\n\n\nspecified character width.");
assert_roundtrips("exact  len", /* exact = */ true);
assert_roundtrips("exact  len\nand then\nsome", /* exact = */ true);
assert_roundtrips("One\ntwo\nthree\nintermingled\n\nlines\nand so", /* exact = */ false);
