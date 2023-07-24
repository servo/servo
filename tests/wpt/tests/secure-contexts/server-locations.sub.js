var https_dir = "https://{{location[hostname]}}:{{ports[https][0]}}{{location[path]}}";
https_dir = https_dir.substr(0, https_dir.lastIndexOf("/") + 1);

var http_dir = "http://{{location[hostname]}}:{{ports[http][0]}}{{location[path]}}";
http_dir = http_dir.substr(0, http_dir.lastIndexOf("/") + 1);

var https_dir2 = "https://{{domains[www]}}:{{ports[https][0]}}{{location[path]}}";
https_dir2 = https_dir2.substr(0, https_dir2.lastIndexOf("/") + 1);

var https_dir3 = "https://{{domains[www1]}}:{{ports[https][0]}}{{location[path]}}";
https_dir3 = https_dir3.substr(0, https_dir3.lastIndexOf("/") + 1);

var https_dir4 = "https://{{domains[www2]}}:{{ports[https][0]}}{{location[path]}}";
https_dir4 = https_dir4.substr(0, https_dir4.lastIndexOf("/") + 1);

var https_dir5 = "https://{{domains[élève]}}:{{ports[https][0]}}{{location[path]}}";
https_dir5 = https_dir5.substr(0, https_dir5.lastIndexOf("/") + 1);
