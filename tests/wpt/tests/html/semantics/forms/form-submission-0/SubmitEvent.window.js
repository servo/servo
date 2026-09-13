// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#the-submitevent-interface

test(() => {
  assert_throws_js(TypeError, () => SubmitEvent(""), "Calling SubmitEvent constructor without 'new' must throw");
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

promise_test(async t => {
  const host = document.body.appendChild(document.createElement('div'));
  t.add_cleanup(() => host.remove());
  const root = host.attachShadow({mode: 'open'});
  const form = document.createElement('form');
  form.id = 'form';
  const button = document.createElement('button');
  button.setAttribute('form', form.id);
  root.append(form, button);

  const event = await new Promise(resolve => {
    form.addEventListener('submit', event => {
      event.preventDefault();
      resolve(event);
    }, {once: true});
    button.click();
  });
  await new Promise(resolve => t.step_timeout(resolve, 0));

  assert_equals(event.submitter, button);
  assert_true(event.composed);
}, 'SubmitEvent.submitter remains set after dispatch in a shadow tree');
