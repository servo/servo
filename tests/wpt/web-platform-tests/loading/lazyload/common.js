// Helper to access the element, its associated loading promise, and also to
// resolve the promise.
class ElementLoadPromise {
  constructor(element_id) {
    this.element_id = element_id;
    this.promise = new Promise((resolve, reject) => {
      this.resolve = resolve
      this.reject = reject
    });
  }
  element() {
    return document.getElementById(this.element_id);
  }
}
