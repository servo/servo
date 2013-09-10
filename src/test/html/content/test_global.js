is(window, window.window);
is(window, this);
for (var key in this) {
  is(this[key], window[key]);
}
finish();