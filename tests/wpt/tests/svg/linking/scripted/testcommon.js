/**
 * The const var for SVG and xlink namespaces
 */
const SVGNS = 'http://www.w3.org/2000/svg';
const XLINKNS = 'http://www.w3.org/1999/xlink';

/**
 * Appends a svg element to the parent.
 *
 * @param test The testharness.js Test object. If provided, this will be used
 *             to register a cleanup callback to remove the div when the test
 *             finishes.
 * @param tag The element tag name.
 * @param parent The parent element of this new created svg element.
 * @param attrs  A dictionary object with attribute names and values to set on
 *               the div.
 */
function createSVGElement(test, tag, parent, attrs) {
  var elem = document.createElementNS(SVGNS, tag);
  if (attrs) {
    for (var attrName in attrs) {
      elem.setAttribute(attrName, attrs[attrName]);
    }
  }
  parent.appendChild(elem);
  if (test) {
    test.add_cleanup(function() {
      elem.remove();
    });
  }
  return elem;
}

/**
 * Create a Promise object which resolves when a specific event fires.
 *
 * @param object The event target.
 * @param name The event name.
 */
function waitEvent(object, name) {
  return new Promise(function(resolve) {
    object.addEventListener(name, resolve, { once: true });
  });
}
