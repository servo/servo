/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ /**
 * Base path for resources. The default value is correct for non-worker WPT, but standalone and
 * workers must access resources using a different base path, so this is overridden in
 * `test_worker-worker.ts` and `standalone.ts`.
 */let baseResourcePath = './resources';let crossOriginHost = '';

function getAbsoluteBaseResourcePath(path) {
  // Path is already an absolute one.
  if (path[0] === '/') {
    return path;
  }

  // Path is relative
  const relparts = window.location.pathname.split('/');
  relparts.pop();
  const pathparts = path.split('/');

  let i;
  for (i = 0; i < pathparts.length; ++i) {
    switch (pathparts[i]) {
      case '':
        break;
      case '.':
        break;
      case '..':
        relparts.pop();
        break;
      default:
        relparts.push(pathparts[i]);
        break;
    }
  }

  return relparts.join('/');
}

function runningOnLocalHost() {
  const hostname = window.location.hostname;
  return hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '::1';
}

/**
 * Get a path to a resource in the `resources` directory relative to the current execution context
 * (html file or worker .js file), for `fetch()`, `<img>`, `<video>`, etc but from cross origin host.
 * Provide onlineUrl if the case running online.
 * @internal MAINTENANCE_TODO: Cases may run in the LAN environment (not localhost but no internet
 * access). We temporarily use `crossOriginHost` to configure the cross origin host name in that situation.
 * But opening to  auto-detect mechanism or other solutions.
 */
export function getCrossOriginResourcePath(pathRelativeToResourcesDir, onlineUrl = '') {
  // A cross origin host has been configured. Use this to load resource.
  if (crossOriginHost !== '') {
    return (
      crossOriginHost +
      getAbsoluteBaseResourcePath(baseResourcePath) +
      '/' +
      pathRelativeToResourcesDir);

  }

  // Using 'localhost' and '127.0.0.1' trick to load cross origin resource. Set cross origin host name
  // to 'localhost' if case is not running in 'localhost' domain. Otherwise, use '127.0.0.1'.
  // host name to locahost unless the server running in
  if (runningOnLocalHost()) {
    let crossOriginHostName = '';
    if (location.hostname === 'localhost') {
      crossOriginHostName = 'http://127.0.0.1';
    } else {
      crossOriginHostName = 'http://localhost';
    }

    return (
      crossOriginHostName +
      ':' +
      location.port +
      getAbsoluteBaseResourcePath(baseResourcePath) +
      '/' +
      pathRelativeToResourcesDir);

  }

  return onlineUrl;
}

/**
 * Get a path to a resource in the `resources` directory, relative to the current execution context
 * (html file or worker .js file), for `fetch()`, `<img>`, `<video>`, etc. Pass the cross origin host
 * name if wants to load resoruce from cross origin host.
 */
export function getResourcePath(pathRelativeToResourcesDir) {
  return baseResourcePath + '/' + pathRelativeToResourcesDir;
}

/**
 * Set the base resource path (path to the `resources` directory relative to the current
 * execution context).
 */
export function setBaseResourcePath(path) {
  baseResourcePath = path;
}

/**
 * Set the cross origin host and cases related to cross origin
 * will load resource from the given host.
 */
export function setCrossOriginHost(host) {
  crossOriginHost = host;
}