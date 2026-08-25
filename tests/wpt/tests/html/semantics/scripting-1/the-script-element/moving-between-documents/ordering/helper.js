function runDelayEventTest(description) {
  const t_original = async_test(description +
      ' still delay the load event in the original Document after move');
  const t_new = async_test(description +
      ' does not delay the load event in the new Document after move');
  const iframe = document.createElement('iframe');
  iframe.setAttribute('src', 'delay-load-event-iframe.html');
  const start_time = performance.now();
  document.body.appendChild(iframe);

  window.onload = t_original.step_func_done(() => {
    // The `#to-be-moved` script should delay the load event until it is loaded
    // (i.e. 3 seconds), not just until it is moved out to another Document
    // (i.e. 1 second). Here we expect the delay should be at least 2 seconds,
    // as the latency can be slightly less than 3 seconds due to preloading.
    assert_greater_than(performance.now() - start_time, 2000,
        'Load event should be delayed until script is loaded');
  });

  window.onloadIframe = t_new.step_func_done(() => {
    // The iframe's load event is fired after 2 seconds of its subresource
    // loading, and shouldn't wait for the `#to-be-moved` script.
    assert_less_than(performance.now() - start_time, 3000,
        'Load event should not be delayed until moved script is loaded');
  });

  t_original.step_timeout(() => {
    const script = document.querySelector('#to-be-moved');
    iframe.contentDocument.body.appendChild(script);
  }, 1000);
}
