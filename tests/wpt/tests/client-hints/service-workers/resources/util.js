async function ch_sw_test(t, worker, url, response) {
  r = await service_worker_unregister_and_register(t, worker, url);
  await wait_for_state(t, r.installing, 'activated')
  var popup_window = window.open("/common/blank.html");
  assert_not_equals(popup_window, null, "Popup windows not allowed?");

  t.add_cleanup(async _=>{
    popup_window.close();
    await r.unregister();
  });

  popup_load = new Promise((resolve, reject) => {
    popup_window.addEventListener('load', t.step_func((e) => {
      if(popup_window.location.pathname != "/blank.html") {
        assert_equals(popup_window.document.body.textContent, response);
        resolve();
      }
    }))
  });

  popup_window.location = url;
  await popup_load;
}
