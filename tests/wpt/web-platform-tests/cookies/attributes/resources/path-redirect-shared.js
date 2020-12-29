window.expireCookie = (cookie) => {
  const cookies = Array.isArray(cookie) ? cookie : [cookie];
  for (let c of cookies) {
    document.cookie = c += "; max-age=0";
  }
}
window.getCookies = () => document.cookie;