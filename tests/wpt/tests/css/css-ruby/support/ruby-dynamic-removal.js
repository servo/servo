function getElements(className) {
  return Array.from(document.getElementsByClassName(className));
}
window.onload = function() {
  // Force a reflow before any changes.
  document.body.clientWidth;

  getElements('remove').forEach(function(e) {
    e.remove();
  });
  getElements('remove-after').forEach(function(e) {
    e.parentNode.removeChild(e.nextSibling);
  });
};
