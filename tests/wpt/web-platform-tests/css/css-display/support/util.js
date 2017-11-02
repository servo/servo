function eachDisplayContentsElementIn(document, window, callbackDo, callbackUndo) {
  var elements = [];

  document.body.offsetHeight;

  // NOTE: Doing qsa('*') and getComputedStyle is just for the
  // test's sake, since it's easier to mess it up when
  // getComputedStyle is involved.
  var all = document.querySelectorAll('*');
  for (var i = 0; i < all.length; ++i) {
    if (window.getComputedStyle(all[i]).display === "contents") {
      callbackDo(all[i]);
      elements.push(all[i]);
    }
  }

  document.body.offsetHeight;

  for (var i = 0; i < elements.length; ++i)
    callbackUndo(elements[i]);

  document.body.offsetHeight;
}
