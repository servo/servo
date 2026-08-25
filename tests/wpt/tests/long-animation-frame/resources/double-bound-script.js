function sensitiveFunction(busy_wait) {
  busy_wait();
}
window.doubleBound = sensitiveFunction.bind(null).bind(null);
