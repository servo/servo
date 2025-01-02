test(() => {
  const frame = document.createElement("iframe"),
        name = "A",
        name2 = "B";
  frame.setAttribute("name", name);
  document.body.append(frame);
  const frameW = frame.contentWindow;
  assert_equals(frameW.name, name);
  frameW.name = name2;
  assert_equals(frame.getAttribute("name"), name);
  assert_equals(frameW.name, name2);
  frame.remove();
  assert_equals(frame.getAttribute("name"), name);
  assert_equals(frameW.name, "");
  frameW.name = name2;
  assert_equals(frame.getAttribute("name"), name);
  assert_equals(frameW.name, "");
}, "Window object's name IDL attribute");
