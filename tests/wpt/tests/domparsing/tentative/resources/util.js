function prepare_html_partial_update(target_type, ref_type, pos, t) {
  const div = document.createElement("div");
  document.body.append(div);
  t.add_cleanup(() => div.remove());
  let target = div;
  if (target_type === "ShadowRoot") {
    const shadow_root = div.attachShadow({ mode: "open" });
    target = shadow_root;
  }

  let ref = document.createElement("span");
  switch (ref_type) {
    case "Element":
      ref.textContent = "ref;";
      break;
    case "Comment":
      ref = document.createComment("ref;");
      break;
    case "Text":
      ref = document.createTextNode("ref;");
      break;
    case "ProcessingInstruction":
      ref = document.createProcessingInstruction("ref", "");
      break;
    default:
      throw new Error("Invalid ref_type");
  }
  target.append(ref);
  const object = ["append", "prepend"].includes(pos) ? target : ref;
  return { target, ref, object };
}


function check_position(target, pos, ref) {
  switch (pos) {
    case "before":
      assert_equals(target.firstChild.textContent, "html;");
      assert_equals(target.firstChild.nextSibling, ref);
      break;
    case "prepend":
      assert_equals(target.textContent, "html;ref;");
      break;
    case "after":
      assert_equals(target.lastChild.textContent, "html;");
      assert_equals(target.firstChild, ref);
      break;
    case "append":
      assert_equals(target.textContent, "ref;html;");
      break;
    case "replaceWith":
      assert_equals(target.textContent, "html;");
      assert_equals(target.firstChild, target.lastChild);
      break;
  }
}