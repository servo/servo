// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#the-formdataevent-interface

test(() => {
  let fd = new FormData();
  assert_throws_js(TypeError, () => { new FormDataEvent() }, '0 arguments');
  assert_throws_js(TypeError, () => { new FormDataEvent('foo') }, '1 argument');
  assert_throws_js(TypeError, () => { new FormDataEvent(fd, fd) }, '2 invalid arguments');
  assert_throws_js(TypeError, () => { new FormDataEvent('foo', null) }, 'Null dictionary');
  assert_throws_js(TypeError, () => { new FormDataEvent('foo', undefined) }, 'Undefined dictionary');
  assert_throws_js(TypeError, () => { new FormDataEvent('foo', { formData: null }) }, 'Null formData');
  assert_throws_js(TypeError, () => { new FormDataEvent('foo', { formData: undefined }) }, 'Undefined formData');
  assert_throws_js(TypeError, () => { new FormDataEvent('foo', { formData: 'bar' }) }, 'Wrong type of formData');
}, 'Failing FormDataEvent constructor');

test(() => {
  let fd = new FormData();
  let event = new FormDataEvent('bar', { formData: fd, bubbles: true });
  assert_equals(event.formData, fd);
  assert_true(event.bubbles);
}, 'Successful FormDataEvent constructor');
