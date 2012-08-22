function setWidth(w, i) {
  var elem = document.documentElement.firstChild;
  elem.width = w;
  debug(elem.width);
  w += i;
  if (w == 0 || w == 1000)
    i *= -1;
  window.setTimeout(function() { setWidth(w, i); }, 50);
}

var elem = document.documentElement.firstChild;
debug(elem.tagName);
debug(elem instanceof HTMLImageElement);
debug(elem.width);
setWidth(1000, -10);