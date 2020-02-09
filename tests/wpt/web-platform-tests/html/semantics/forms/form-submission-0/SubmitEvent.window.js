// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#the-submitevent-interface

test(() => {
  let button = document.createElement('button');
  assert_throws_js(TypeError, () => { new SubmitEvent() }, '0 arguments');
  assert_throws_js(TypeError, () => { new SubmitEvent('foo', { submitter: 'bar' }) }, 'Wrong type of submitter');
}, 'Failing SubmitEvent constructor');

test(() => {
  let button = document.createElement('button');
  let event = new SubmitEvent('bar', { submitter: button, bubbles: true });
  assert_equals(event.submitter, button);
  assert_true(event.bubbles);
}, 'Successful SubmitEvent constructor');

test(() => {
  let event1 = new SubmitEvent('bar', {submitter: null});
  assert_equals(event1.submitter, null);
  let event2 = new SubmitEvent('baz', {submitter: undefined});
  assert_equals(event2.submitter, null);
}, 'Successful SubmitEvent constructor; null/undefined submitter');

test(() => {
  let event1 = new SubmitEvent('bar', null);
  assert_equals(event1.submitter, null);
  let event2 = new SubmitEvent('baz', undefined);
  assert_equals(event2.submitter, null);
}, 'Successful SubmitEvent constructor; null/undefined dictionary');

test(() => {
  let event1 = new SubmitEvent('bar', {});
  assert_equals(event1.submitter, null);
  let button = document.createElement('button');
  let event2 = new SubmitEvent("bax", button);
  assert_equals(event2.submitter, null);
}, 'Successful SubmitEvent constructor; empty dictionary');

test(() => {
  let event = new SubmitEvent('bar');
  assert_equals(event.submitter, null);
}, 'Successful SubmitEvent constructor; missing dictionary');
