function header(name, value) {
  return `header(${name},${value})`;
}

function contentType(type) {
  return header("Content-Type", type);
}

function contentTypeOptions(type) {
  return header("X-Content-Type-Options", type);
}

function fetchORB(file, options, ...pipe) {
  return fetch(`${file}${pipe.length ? `?pipe=${pipe.join("|")}` : ""}`, {
    ...(options || {}),
    mode: "no-cors",
  });
}
