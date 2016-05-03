if (typeof aa != 'undefined') {
  alert_assert(aa);
} else {
  alert_assert("Failed - allowed inline script blocked by meta policy outside head.");
}
