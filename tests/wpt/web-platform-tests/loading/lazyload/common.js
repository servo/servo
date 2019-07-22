// Helper to access the element, its associated loading promise, and also to
// resolve the promise.
class ElementLoadPromise {
  constructor(element_id) {
    this.element_id = element_id;
    this.promise = new Promise(resolve => {
      this.resolve = resolve
    });
  }
  element() {
    return document.getElementById(this.element_id);
  }
}
