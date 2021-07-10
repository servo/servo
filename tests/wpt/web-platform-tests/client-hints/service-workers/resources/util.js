async function ch_sw_test(t, worker, url, response) {
  r = await service_worker_unregister_and_register(t, worker, worker)
  await wait_for_state(t, r.installing, 'activated')
  var popup_window = window.open('about:blank');
  assert_not_equals(popup_window, null, "Popup windows not allowed?");

  t.add_cleanup(async _=>{
    popup_window.close();
    await r.unregister();
  });

  popup_window.addEventListener('load', (e) => {
    t.step(()=>{assert_equals(popup_window.document.body.textContent, response)});
    t.done();
  });

  popup_window.location = url;
}
