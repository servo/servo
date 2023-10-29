// Keep this mostly synchronized with
// html/semantics/forms/attributes-common-to-form-controls/dirname-only-if-applies.html
// except that won't have "reset" and "button" as those don't submit their value
[
  "hidden",
  "text",
  "search",
  "tel",
  "url",
  "email",
  "password",
  "submit",
  "reset",
  "button"
].forEach(type => {
  test(t => {
    const input = document.createElement("input");
    t.add_cleanup(() => input.remove());
    input.type = type;
    assert_equals(input.type, type);
    input.dir = "auto";
    input.value = "\u05D0"; // The Hebrew letter Alef (strongly RTL)
    document.body.append(input);
    assert_true(input.matches(":dir(rtl)"));
  }, `<input dir=auto type=${type}> directionality`);
});

[
  "date",
  "month",
  "week",
  "time",
  "datetime-local",
  "number",
  "range",
  "color",
  "checkbox",
  "radio",
  // "file" // value setter throws
  "image"
].forEach(type => {
  test(t => {
    const input = document.createElement("input");
    t.add_cleanup(() => input.remove());
    input.type = type;
    assert_equals(input.type, type);
    input.dir = "auto";
    input.value = "\u05D0"; // The Hebrew letter Alef (strongly RTL)
    document.body.append(input);
    assert_true(input.matches(":dir(ltr)"));
  }, `<input dir=auto type=${type}> directionality`);
});

test(t => {
  const input = document.createElement("textarea");
  t.add_cleanup(() => input.remove());
  input.dir = "auto";
  input.value = "\u05D0"; // The Hebrew letter Alef (strongly RTL)
  document.body.append(input);
  assert_true(input.matches(":dir(rtl)"));
}, `<textarea dir=auto> directionality`);
