const run_in_fenced_frame = (func_name, description, is_nested) => {
  promise_test(async test => {
    const key = token();
    const url = is_nested ?
                'resources/sandboxed-features-looser-restriction.sub.html?' :
                'resources/sandboxed-features-inner.sub.html?';
    let params = new URLSearchParams();
    params.set('key', key);
    params.set('test_func', func_name);
    const frame = document.createElement('fencedframe');
    const frame_url = 'resources/sandboxed-features-inner.sub.html?' +
    params.toString();
    const config = new FencedFrameConfig(generateURL(frame_url, []));
    frame.config = config;
    test.add_cleanup(() => {
      frame.remove();
    });
    document.body.appendChild(frame);
    assert_equals(await nextValueFromServer(key), 'done');
  }, description);
};

const run_sanboxed_feature_test = (func_name, description) => {
  run_in_fenced_frame(func_name, description, false);
  run_in_fenced_frame(func_name, description + '[looser sandboxed]', true);
};

async function test_prompt() {
  assert_equals(
    window.prompt('Test prompt'),
    null,
    'window.prompt() must synchronously return null in a fenced frame without' +
    ' blocking on user input.');
}

async function test_alert() {
  assert_equals(
    window.alert('Test alert'),
    undefined,
    'window.alert() must synchronously return undefined in a fenced frame' +
    '  without blocking on user input.');
}

async function test_confirm() {
  assert_equals(
    window.confirm('Test confirm'),
    false,
    'window.confirm() must synchronously return false in a fenced frame' +
    ' without blocking on user input.');
}

async function test_print() {
  assert_equals(
    window.print(),
    undefined,
    'window.print() must synchronously return undefined in a fenced frame' +
    ' without blocking on user input.');

  assert_equals(
    document.execCommand('print', false, null),
    false,
    'execCommand(\'print\') must synchronously return false in a fenced frame' +
    ' without blocking on user input.');
}

async function test_document_domain() {
  assert_throws_dom('SecurityError', () => {
    document.domain = 'example.test';
  });
  assert_throws_dom('SecurityError', () => {
    document.domain = document.domain;
  });
  assert_throws_dom('SecurityError', () => {
    (new Document).domain = document.domain;
  });
  assert_throws_dom('SecurityError', () => {
    document.implementation.createHTMLDocument().domain = document.domain;
  });
  assert_throws_dom('SecurityError', () => {
    document.implementation.createDocument(null, '').domain = document.domain;
  });
  assert_throws_dom('SecurityError', () => {
    document.createElement('template').content.ownerDocument.domain =
        document.domain;
  });
}

async function test_presentation_request() {
  assert_throws_dom('SecurityError', () => {
    new PresentationRequest([location.href]);
  });
}

async function test_screen_orientation_lock() {
  try {
    await screen.orientation.lock('portrait');
  } catch (e) {
    assert_equals(
      e.name,
      'SecurityError',
      'orientation.lock() must throw a SecurityError in a fenced frame.');
    return;
  }
  assert_unreached('orientation.lock() must throw an error');
}

async function test_pointer_lock() {
  await simulateGesture();

  const canvas = document.createElement('canvas');
  document.body.appendChild(canvas);
  const pointerlockerror_promise = new Promise(resolve => {
    document.addEventListener('pointerlockerror', resolve);
  });
  try {
    await canvas.requestPointerLock();
  } catch (e) {
    assert_equals(
      e.name,
      'SecurityError',
      'orientation.lock() must throws a SecurityError in a fenced frame.');
    await pointerlockerror_promise;
    return;
  }
  assert_unreached('requestPointerLock() must fail in a fenced frame');
}
