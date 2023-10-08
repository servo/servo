function html_direction(element) {
  let is_ltr = element.matches(":dir(ltr)");
  let is_rtl = element.matches(":dir(rtl)");
  if (is_ltr == is_rtl) {
    return "error";
  }
  return is_ltr ? "ltr" : "rtl";
}
