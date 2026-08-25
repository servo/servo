function base64urlEncode(data) {
  let result = btoa(data);
  return result.replace(/=+$/g, '').replace(/\+/g, "-").replace(/\//g, "_");
}

const sha256ofURL = async (url) => {
  const buffer = new TextEncoder().encode(url.toString());
  const hashBuffer = await window.crypto.subtle.digest('SHA-256', buffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const arr = hashArray.map(b => String.fromCharCode(b)).join('');
  return base64urlEncode(arr);
};
