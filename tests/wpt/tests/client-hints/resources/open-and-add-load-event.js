function open_and_add_load_event(href) {
  return new Promise((resolve) => {
    // While not practically possible, opening "blank" first and setting the
    // href after allows for the theoretical possibility of registering the event
    // after the window is loaded.
    let popup_window = window.open("/resources/blank.html");
    assert_not_equals(popup_window, null, "Popup windows not allowed?");
    popup_window.addEventListener('load', resolve, {once: true});
    popup_window.location.href = href;
  });
}

async function open_and_expect_headers(href) {
  let e = await new Promise(resolve => {
    let popup_window = window.open("/resources/blank.html");
    assert_not_equals(popup_window, null, "Popup windows not allowed?");
    window.addEventListener('message', resolve, false);
    popup_window.location.href = href;
  });

  assert_equals(e.data, "PASS");
  return e;
}