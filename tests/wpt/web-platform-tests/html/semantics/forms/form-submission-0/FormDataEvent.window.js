// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#the-formdataevent-interface

test(() => {
  let fd = new FormData();
  let typeError = new TypeError();
  assert_throws(typeError, () => { new FormDataEvent() }, '0 arguments');
  assert_throws(typeError, () => { new FormDataEvent('foo') }, '1 argument');
  assert_throws(typeError, () => { new FormDataEvent(fd, fd) }, '2 invalid arguments');
  assert_throws(typeError, () => { new FormDataEvent('foo', null) }, 'Null dictionary');
  assert_throws(typeError, () => { new FormDataEvent('foo', undefined) }, 'Undefined dictionary');
  assert_throws(typeError, () => { new FormDataEvent('foo', { formData: null }) }, 'Null formData');
  assert_throws(typeError, () => { new FormDataEvent('foo', { formData: undefined }) }, 'Undefined formData');
  assert_throws(typeError, () => { new FormDataEvent('foo', { formData: 'bar' }) }, 'Wrong type of formData');
}, 'Failing FormDataEvent constructor');

test(() => {
  let fd = new FormData();
  let event = new FormDataEvent('bar', { formData: fd, bubbles: true });
  assert_equals(event.formData, fd);
  assert_true(event.bubbles);
}, 'Successful FormDataEvent constructor');
