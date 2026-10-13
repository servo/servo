function html_direction(element) {
  let is_ltr = element.matches(":dir(ltr)");
  let is_rtl = element.matches(":dir(rtl)");
  if (is_ltr == is_rtl) {
    return "error";
  }
  return is_ltr ? "ltr" : "rtl";
}

function setup_tree(light_tree, shadow_tree, shadowOptions = {}) {
  let body = document.body;
  let old_length = body.childNodes.length;
  body.insertAdjacentHTML("beforeend", light_tree.trim());
  if (body.childNodes.length != old_length + 1) {
    throw "unexpected markup";
  }
  let result = body.lastChild;

  if (!shadowOptions.mode) {
    shadowOptions.mode = "open";
  }
  if (shadow_tree) {
    let shadow = result.querySelector("#root").attachShadow(shadowOptions);
    shadow.innerHTML = shadow_tree.trim();
    return [result, shadow];
  }

  return result;
}
