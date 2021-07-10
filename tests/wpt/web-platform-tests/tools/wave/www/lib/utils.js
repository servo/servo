const utils = {
  parseQuery: queryString => {
    if (queryString.indexOf("?") === -1) return {};
    queryString = queryString.split("?")[1];
    const query = {};
    for (let part of queryString.split("&")) {
      const keyValue = part.split("=");
      query[keyValue[0]] = keyValue[1] ? keyValue[1] : null;
    }
    return query;
  },
  percent: (count, total) => {
    const percent = Math.floor((count / total) * 10000) / 100;
    if (!percent) {
      return 0;
    }
    return percent;
  },
  saveBlobAsFile: (blob, filename) => {
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.style.display = "none";
    document.body.appendChild(a);
    a.href = url;
    a.download = filename;
    a.click();
    document.body.removeChild(a);
  },
  millisToTimeString(totalMilliseconds) {
    let milliseconds = (totalMilliseconds % 1000) + "";
    milliseconds = milliseconds.padStart(3, "0");
    let seconds = (Math.floor(totalMilliseconds / 1000) % 60) + "";
    seconds = seconds.padStart(2, "0");
    let minutes = (Math.floor(totalMilliseconds / 60000) % 60) + "";
    minutes = minutes.padStart(2, "0");
    let hours = Math.floor(totalMilliseconds / 3600000) + "";
    hours = hours.padStart(2, "0");
    return `${hours}:${minutes}:${seconds}`;
  },
  getBrowserIcon(browser) {
    switch (browser.toLowerCase()) {
      case "firefox":
        return "fab fa-firefox";
      case "edge":
        return "fab fa-edge";
      case "chrome":
      case "chromium":
        return "fab fa-chrome";
      case "safari":
      case "webkit":
        return "fab fa-safari";
    }
  },
  copyObject(object) {
    return JSON.parse(JSON.stringify(object));
  }
};
