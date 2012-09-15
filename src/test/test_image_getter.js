function setWidth(w, i) {
  elem.width = w;
  window.alert(elem.width);
  w += i;
  if (w == 0 || w == 1000)
    i *= -1;
  window.setTimeout(function() { setWidth(w, i); }, 50);
}

var elem = document.documentElement.firstChild.firstChild.nextSibling.firstChild;
window.alert(elem.tagName);
window.alert(elem instanceof HTMLImageElement);
window.alert(elem instanceof HTMLElement);
window.alert(elem instanceof Element);
window.alert(elem.width);
setWidth(1000, -10);