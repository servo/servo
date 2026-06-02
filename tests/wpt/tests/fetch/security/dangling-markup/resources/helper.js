function assert_no_message_from_frame(test, frame) {
  window.addEventListener("message", test.step_func(e => {
    assert_not_equals(e.source, frame.contentWindow);
  }));
}

function appendFrameAndGetElement(test, frame) {
  return new Promise((resolve, reject) => {
    frame.onload = test.step_func(_ => {
      frame.onload = null;
      resolve(frame.contentDocument.querySelector('#dangling'));
    });
    document.body.appendChild(frame);
  });
}

function appendAndSubmit(test, frame) {
  return new Promise((resolve, reject) => {
    frame.onload = test.step_func(_ => {
      frame.onload = null;
      frame.contentDocument.querySelector('form').addEventListener("error", _ => {
        resolve("error");
      });
      frame.contentDocument.querySelector('form').addEventListener("submit", _ => {
        resolve("submit");
      });
      frame.contentDocument.querySelector('[type=submit]').click();
    });
    document.body.appendChild(frame);
  });
}

function assert_no_submission(test, frame) {
  assert_no_message_from_frame(test, frame);

  appendAndSubmit(test, frame)
    .then(test.step_func_done(result => {
      assert_equals(result, "error");
      frame.remove();
    }));
}

function assert_img_loaded(test, frame) {
  appendFrameAndGetElement(test, frame)
    .then(test.step_func_done(img => {
      assert_equals(img.naturalHeight, 103, "Height");
      assert_equals(img.naturalWidth, 76, "Width");
    }));
}

function assert_img_not_loaded(test, frame) {
  appendFrameAndGetElement(test, frame)
    .then(test.step_func_done(img => {
      assert_equals(img.naturalHeight, 0, "Height");
      assert_equals(img.naturalWidth, 0, "Width");
    }));
}

function createFrame(markup) {
  var i = document.createElement('iframe');
  i.srcdoc = `${markup}sekrit`;
  return i;
}
