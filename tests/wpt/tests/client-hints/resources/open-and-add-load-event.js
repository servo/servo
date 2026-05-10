function open_and_add_load_event(href) {
  return new Promise((resolve) => {
    let popup_window = window.open(href);
    assert_not_equals(popup_window, null, "Popup windows not allowed?");
    popup_window.addEventListener('load', resolve, {once: true});
  });
}

async function open_and_expect_headers(href) {
  let e = await new Promise(resolve => {
    let popup_window = window.open(href);
    assert_not_equals(popup_window, null, "Popup windows not allowed?");
    window.addEventListener('message', resolve, false);
  });

  assert_equals(e.data, "PASS");
  return e;
}