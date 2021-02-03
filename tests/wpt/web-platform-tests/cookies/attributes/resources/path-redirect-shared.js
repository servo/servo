function expireCookie(cookie) {
  const cookies = Array.isArray(cookie) ? cookie : [cookie];
  for (let c of cookies) {
    document.cookie = c += "; max-age=0";
  }
}

function getCookies() {
  return document.cookie;
}

window.addEventListener("message", (e) => {
  if (e.data == "getCookies") {
    const cookies = getCookies();
    e.source.postMessage({"cookies": cookies}, '*');
  }

  if (typeof e.data == "object" && 'expireCookie' in e.data) {
    expireCookie(e.data.expireCookie);
    e.source.postMessage("expired", '*');
  }
});