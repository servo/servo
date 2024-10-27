'use strict';

function getCookieMessage(cookie_string, prefix, suffix, expected_id) {
  let message = "";
  if (cookie_string.includes(prefix + "Strict" + suffix + "=" + expected_id)) {
    message += prefix + "Strict" + suffix + "-";
  }
  if (cookie_string.includes(prefix + "Lax" + suffix + "=" + expected_id)) {
    message += prefix + "Lax" + suffix + "-";
  }
  if (cookie_string.includes(prefix + "None" + suffix + "=" + expected_id)) {
    message += prefix + "None" + suffix + "-";
  }
  return message;
}
