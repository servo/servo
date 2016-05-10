var https_dir = "https://{{location[hostname]}}:{{ports[https][0]}}{{location[path]}}";
https_dir = https_dir.substr(0, https_dir.lastIndexOf("/") + 1);

var http_dir = "http://{{location[hostname]}}:{{ports[http][0]}}{{location[path]}}";
http_dir = http_dir.substr(0, http_dir.lastIndexOf("/") + 1);

