const el = document.createElement("base");
el.href = "{{GET[baseurl]}}";
document.currentScript.after(el);
