/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ let windowURL = undefined;
function getWindowURL() {
  if (windowURL === undefined) {
    windowURL = new URL(window.location.toString());
  }
  return windowURL;
}

export function optionEnabled(opt, searchParams = getWindowURL().searchParams) {
  const val = searchParams.get(opt);
  return val !== null && val !== '0';
}

export function optionString(opt, searchParams = getWindowURL().searchParams) {
  return searchParams.get(opt) || '';
}
