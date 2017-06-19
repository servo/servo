function assert_usb_available_in_iframe(test, origin, expected) {
  let frame = document.createElement('iframe');
  frame.src = origin + '/webusb/resources/check-availability.html';

  window.addEventListener('message', test.step_func(evt => {
    if (evt.source == frame.contentWindow) {
      assert_equals(evt.data, expected);
      document.body.removeChild(frame);
      test.done();
    }
  }));

  document.body.appendChild(frame);
}
