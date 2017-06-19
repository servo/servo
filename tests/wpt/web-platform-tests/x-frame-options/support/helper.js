function assert_no_message_from(frame, test) {
  wait_for_message_from(frame, test)
    .then(test.unreached_func("Frame should not have sent a message."));
}

function wait_for_message_from(frame, test) {
  return new Promise((resolve, reject) => {
    window.addEventListener("message", test.step_func(e => {
      if (e.source == frame.contentWindow)
        resolve(e);
    }));
  });
}
