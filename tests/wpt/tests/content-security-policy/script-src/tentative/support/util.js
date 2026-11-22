const sha256ofURLBytes = async (url) => {
  const buffer = new TextEncoder().encode(url.toString());
  return new Uint8Array(await window.crypto.subtle.digest('SHA-256', buffer));
};

// The hash of the URL in the format of url hashes in a policy, i.e. base64url
// without padding.
const sha256ofURL = async (url) => {
  return (await sha256ofURLBytes(url)).toBase64({
    alphabet: "base64url",
    omitPadding: true,
  });
};

// The hash of the URL in the format used by the urlHash member of
// SecurityPolicyViolationEvent, i.e. standard base64 with padding.
const sha256ofURLForReport = async (url) => {
  return (await sha256ofURLBytes(url)).toBase64();
};
