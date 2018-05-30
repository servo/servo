function loadScript(url, { doc = document }={}) {
  return new Promise((resolve, reject) => {
    const script = doc.createElement('script');
    script.onload = () => resolve();
    script.onerror = () => reject(Error("Script load failed"));
    script.src = url;
    doc.body.appendChild(script);
  })
}
