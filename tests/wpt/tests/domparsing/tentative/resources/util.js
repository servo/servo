function prepare_html_partial_update(target_type, ref_type, pos, t) {
  let target;
  let template;
  if (target_type === "template.content") {
    template = document.createElement("template");
    target = template.content;
  } else {
    const div = document.createElement("div");
    document.body.append(div);
    t.add_cleanup(() => div.remove());
    target = div;
    if (target_type === "ShadowRoot") {
      target = div.attachShadow({ mode: "open" });
    }
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
  let object;
  if (["append", "prepend"].includes(pos)) {
    object = (target_type === "template.content") ? template : target;
  } else {
    object = ref;
  }
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