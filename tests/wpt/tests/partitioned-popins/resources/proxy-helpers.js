'use strict';

function customMethod() {
}

let customAttribute = "";

function getUsableMethods(proxy) {
  let message = "";
  try {
    proxy.closed;
    message += "Closed,"
  } catch (_) {}
  try {
    proxy.blur();
    message += "Blur,"
  } catch (_) {}
  try {
    proxy.onblur;
    message += "OnBlur,"
  } catch (_) {}
  try {
    proxy.opener;
    message += "Opener,"
  } catch (_) {}
  try {
    proxy.length;
    message += "Length,"
  } catch (_) {}
  try {
    proxy.name = "foo";
    message += "Name,"
  } catch (_) {}
  try {
    proxy[0];
    message += "AnonymousIndex,"
  } catch (_) {}
  try {
    proxy['test'];
    message += "AnonymousName,"
  } catch (_) {}
  try {
    proxy.customMethod();
    message += "CustomMethod,"
  } catch (_) {}
  try {
    proxy.customAttribute;
    message += "CustomAttributeGet,"
  } catch (_) {}
  try {
    proxy.customAttribute = "";
    message += "CustomAttributeSet,"
  } catch (_) {}
  if (proxy.then == undefined) {
    message += "Then,"
  }
  return message;
}
