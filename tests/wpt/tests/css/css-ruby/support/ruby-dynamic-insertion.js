window.onload = function() {
  // Force a reflow before any changes.
  document.body.clientWidth;

  var elems = document.querySelectorAll('[data-insert]');
  Array.from(elems).forEach(function(e) {
    var parent, ref;
    switch (e.dataset.insert) {
      case 'start':
        parent = e;
        ref = e.firstChild;
        break;

      case 'end':
        parent = e;
        ref = null;
        break;

      case 'before':
        parent = e.parentNode;
        ref = e;
        break;

      case 'after':
        parent = e.parentNode;
        ref = e.nextSibling;
        break;
    }

    var elem, textnode;
    if ('text' in e.dataset) {
      textnode = document.createTextNode(e.dataset.text);
    }
    if ('tag' in e.dataset) {
      elem = document.createElement(e.dataset.tag);
      if (textnode) {
        elem.appendChild(textnode);
      }
    }
    parent.insertBefore(elem ? elem : textnode, ref);
  });
};
