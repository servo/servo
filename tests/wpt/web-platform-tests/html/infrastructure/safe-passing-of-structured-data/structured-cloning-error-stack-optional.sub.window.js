// META: script=/common/utils.js

// .stack properties on errors are unspecified, but are present in most
// browsers, most of the time. https://github.com/tc39/proposal-error-stacks/ tracks standardizing them.
// Tests will pass automatically if the .stack property isn't present.

stackTests(() => {
  return new Error('some message');
}, 'page-created Error');

stackTests(() => {
  return new DOMException('InvalidStateError', 'some message');
}, 'page-created DOMException');

stackTests(() => {
  try {
    Object.defineProperty();
  } catch (e) {
    return e;
  }
}, 'JS-engine-created TypeError');

stackTests(() => {
  try {
    HTMLParagraphElement.prototype.align;
  } catch (e) {
    return e;
  }
}, 'web API-created TypeError');

stackTests(() => {
  try {
    document.createElement('');
  } catch (e) {
    return e;
  }
}, 'web API-created DOMException');

function stackTests(errorFactory, description) {
  async_test(t => {
    const error = errorFactory();
    const originalStack = error.stack;

    if (!originalStack) {
      t.done();
      return;
    }

    const worker = new Worker('resources/echo-worker.js');
    worker.onmessage = t.step_func_done(e => {
      assert_equals(e.data.stack, originalStack);
    });

    worker.postMessage(error);
  }, description + ' (worker)');

  async_test(t => {
    const thisTestId = token();

    const error = errorFactory();
    const originalStack = error.stack;

    if (!originalStack) {
      t.done();
      return;
    }

    const iframe = document.createElement('iframe');
    window.addEventListener('message', t.step_func(e => {
      if (e.data.testId === thisTestId) {
        assert_equals(e.data.error.stack, originalStack);
        t.done();
      }
    }));

    iframe.onload = t.step_func(() => {
      iframe.contentWindow.postMessage({ error, testId: thisTestId }, "*");
    });

    const crossSiteEchoIFrame = new URL('resources/echo-iframe.html', location.href);
    crossSiteEchoIFrame.hostname = '{{hosts[alt][www1]}}';
    iframe.src = crossSiteEchoIFrame;
    document.body.append(iframe);
  }, description + ' (cross-site iframe)');
}
