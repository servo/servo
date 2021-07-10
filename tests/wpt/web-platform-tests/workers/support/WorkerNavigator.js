(async () => {
  const obj = new Object();
  obj.appName    = navigator.appName;
  obj.appVersion = navigator.appVersion;
  obj.platform   = navigator.platform;
  obj.userAgent  = navigator.userAgent;
  obj.onLine     = navigator.onLine;
  if (navigator.userAgentData) {
    obj.brands = navigator.userAgentData.brands;
    obj.mobile = navigator.userAgentData.mobile;
    const highEntropyValues = await navigator.userAgentData.getHighEntropyValues(["platform", "platformVersion", "architecture", "model", "uaFullVersion"]);
    obj.platform = highEntropyValues.platform;
    obj.platformVersion = highEntropyValues.platformVersion;
    obj.architecture = highEntropyValues.architecture;
    obj.model = highEntropyValues.model;
    obj.uaFullVersion = highEntropyValues.uaFullVersion;
    obj.NavigatorUADataExposed = (typeof self.NavigatorUAData != "undefined");
  }
  postMessage(obj);
})();
