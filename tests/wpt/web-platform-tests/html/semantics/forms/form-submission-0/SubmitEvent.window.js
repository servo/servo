// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#the-submitevent-interface

test(() => {
  let button = document.createElement('button');
  let typeError = new TypeError();
  assert_throws(typeError, () => { new SubmitEvent() }, '0 arguments');
  assert_throws(typeError, () => { new SubmitEvent('bar', button) }, '1 invalid arguments');
  assert_throws(typeError, () => { new SubmitEvent(button, button) }, '2 invalid arguments');
  assert_throws(typeError, () => { new SubmitEvent('foo', null) }, 'Null dictionary');
  assert_throws(typeError, () => { new SubmitEvent('foo', undefined) }, 'Undefined dictionary');
  assert_throws(typeError, () => { new SubmitEvent('foo', { submitter: null }) }, 'Null submitter');
  assert_throws(typeError, () => { new SubmitEvent('foo', { submitter: undefined }) }, 'Undefined submitter');
  assert_throws(typeError, () => { new SubmitEvent('foo', { submitter: 'bar' }) }, 'Wrong type of submitter');
}, 'Failing SubmitEvent constructor');

test(() => {
  let button = document.createElement('button');
  let event = new SubmitEvent('bar', { submitter: button, bubbles: true });
  assert_equals(event.submitter, button);
  assert_true(event.bubbles);
}, 'Successful SubmitEvent constructor');

test(() => {
  let event = new SubmitEvent('bar', { submitter: null});
  assert_equals(event.submitter, null);
}, 'Successful SubmitEvent constructor; null submitter');

test(() => {
  let event = new SubmitEvent('baz', {});
  assert_equals(event.submitter, null);
}, 'Successful SubmitEvent constructor; empty dictionary');

test(() => {
  let event = new SubmitEvent('baz');
  assert_equals(event.submitter, null);
}, 'Successful SubmitEvent constructor; missing dictionary');
