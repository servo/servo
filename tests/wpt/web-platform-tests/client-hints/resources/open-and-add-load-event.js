function open_and_add_load_event(href, func) {
  // While not practically possible, opening "blank" first and setting the
  // href after allows for the theoretical possibility of registering the event
  // after the window is loaded.
  let popup_window = window.open("about:blank");
  assert_not_equals(popup_window, null, "Popup windows not allowed?");
  popup_window.addEventListener('load', func, false);
  popup_window.location.href=href;
}