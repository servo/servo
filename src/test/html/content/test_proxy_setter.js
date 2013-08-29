is(window.document.title, '');
window.document.title = 'foo';
is(window.document.title, 'foo');
finish();