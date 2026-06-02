test(() => {
  const container = document.createElement("div");
  document.body.append(container);
  container.setAttribute("lang", "en-CA");

  const child = document.createElementNS("div", "test");
  container.append(child);
  child.setAttribute("lang", "en-NZ");

  assert_true(container.matches(":lang(en-CA)"), "container matches en-CA");
  assert_true(child.matches(":lang(en-CA)"), "child matches en-CA");
  assert_false(container.matches(":lang(en-NZ)"), "container does not match en-NZ");
  assert_false(child.matches(":lang(en-NZ)"), "child does not match en-NZ");

  container.remove();
}, "unnamespaced lang attribute only works on elements in the HTML namespace");
