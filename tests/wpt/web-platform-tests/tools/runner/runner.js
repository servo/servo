/*jshint nonew: false */
(function() {
"use strict";
var runner;
var testharness_properties = {output:false,
                              timeout_multiplier:1};

function Manifest(path) {
    this.data = null;
    this.path = path;
    this.num_tests = null;
}

Manifest.prototype = {
    load: function(loaded_callback) {
        this.generate(loaded_callback);
    },

    do_load: function(loaded_callback) {
        var xhr = new XMLHttpRequest();
        xhr.onreadystatechange = function() {
            if (xhr.readyState !== 4) {
                return;
            }
            if (!(xhr.status === 200 || xhr.status === 0)) {
                throw new Error("Manifest " + this.path + " failed to load");
            }
            this.data = JSON.parse(xhr.responseText);
            loaded_callback();
        }.bind(this);
        xhr.open("GET", this.path);
        xhr.send(null);
    },

    generate: function(loaded_callback) {
        var xhr = new XMLHttpRequest();
        xhr.onreadystatechange = function() {
            if (xhr.readyState !== 4) {
                return;
            }
            if (!(xhr.status === 200 || xhr.status === 0)) {
                throw new Error("Manifest generation failed");
            }
            this.do_load(loaded_callback);
        }.bind(this);
        xhr.open("POST", "update_manifest.py");
        xhr.send(null);
    },

    by_type:function(type) {
        if (this.data.items.hasOwnProperty(type)) {
            return this.data.items[type];
        } else {
            return [];
        }
    }
};

function ManifestIterator(manifest, path, test_types) {
    this.manifest = manifest;
    this.path = path;
    this.test_types = test_types;
    this.test_types_index = -1;
    this.test_list = null;
    this.test_index = null;
}

ManifestIterator.prototype = {
    next: function() {
        var manifest_item = null;

        if (this.test_types.length === 0) {
            return null;
        }

        while (!manifest_item) {
            while (this.test_list === null || this.test_index >= this.test_list.length) {
                this.test_types_index++;
                if (this.test_types_index >= this.test_types.length) {
                    return null;
                }
                this.test_index = 0;
                this.test_list = this.manifest.by_type(this.test_types[this.test_types_index]);
            }

            manifest_item = this.test_list[this.test_index++];
            while (manifest_item && !this.matches(manifest_item)) {
                manifest_item = this.test_list[this.test_index++];
            }
            if (manifest_item) {
                return this.to_test(manifest_item);
            }
        }
    },

    matches: function(manifest_item) {
        return manifest_item.url.indexOf(this.path) === 0;
    },

    to_test: function(manifest_item) {
        var test = {
            type: this.test_types[this.test_types_index],
            url: manifest_item.url
        };
        if (manifest_item.hasOwnProperty("ref_url")) {
            test.ref_type = manifest_item.ref_type;
            test.ref_url = manifest_item.ref_url;
        }
        return test;
    },

    count: function() {
        return this.test_types.reduce(function(prev, current) {
            var matches = this.manifest.by_type(current).filter(function(x) {
                return this.matches(x);
            }.bind(this));
            return prev + matches.length;
        }.bind(this), 0);
    }
};

function VisualOutput(elem, runner) {
    this.elem = elem;
    this.runner = runner;
    this.results_table = null;
    this.section_wrapper = null;
    this.results_table = this.elem.querySelector(".results > table");
    this.section = null;
    this.manifest_status = this.elem.querySelector("#manifest");
    this.progress = this.elem.querySelector(".summary .progress");
    this.meter = this.progress.querySelector(".progress-bar");
    this.result_count = null;
    this.json_results_area = this.elem.querySelector("textarea");
    this.instructions = document.querySelector(".instructions");

    this.elem.style.display = "none";
    this.runner.manifest_wait_callbacks.push(this.on_manifest_wait.bind(this));
    this.runner.start_callbacks.push(this.on_start.bind(this));
    this.runner.result_callbacks.push(this.on_result.bind(this));
    this.runner.done_callbacks.push(this.on_done.bind(this));
}

VisualOutput.prototype = {
    clear: function() {
        this.result_count = {"PASS":0,
                             "FAIL":0,
                             "ERROR":0,
                             "TIMEOUT":0,
                             "NOTRUN":0};
        for (var p in this.result_count) {
            if (this.result_count.hasOwnProperty(p)) {
                this.elem.querySelector("td." + p).textContent = 0;
            }
        }
        if (this.json_results_area) {
            this.json_results_area.parentNode.removeChild(this.json_results_area);
        }
        this.meter.style.width = '0px';
        this.meter.textContent = '0%';
        this.manifest_status.style.display = "none";
        this.elem.querySelector(".jsonResults").style.display = "none";
        this.results_table.removeChild(this.results_table.tBodies[0]);
        this.results_table.appendChild(document.createElement("tbody"));
    },

    on_manifest_wait: function() {
        this.clear();
        this.instructions.style.display = "none";
        this.elem.style.display = "block";
        this.manifest_status.style.display = "inline";
    },

    on_start: function() {
        this.clear();
        this.instructions.style.display = "none";
        this.elem.style.display = "block";
        this.meter.classList.remove("stopped");
        this.meter.classList.add("progress-striped", "active");
    },

    on_result: function(test, status, message, subtests) {
        var row = document.createElement("tr");

        var subtest_pass_count = subtests.reduce(function(prev, current) {
            return (current.status === "PASS") ? prev + 1 : prev;
        }, 0);

        var subtest_notrun_count = subtests.reduce(function(prev, current) {
            return (current.status === "NOTRUN") ? prev +1 : prev;
        }, 0);

        var subtests_count = subtests.length;

        var test_status;
        if (subtest_pass_count === subtests_count &&
            (status == "OK" || status == "PASS")) {
            test_status = "PASS";
        } else if (subtest_notrun_count == subtests_count) {
            test_status = "NOTRUN";
        } else if (subtests_count > 0 && status === "OK") {
            test_status = "FAIL";
        } else {
            test_status = status;
        }

        subtests.forEach(function(subtest) {
            if (this.result_count.hasOwnProperty(subtest.status)) {
                this.result_count[subtest.status] += 1;
            }
        }.bind(this));
        if (this.result_count.hasOwnProperty(status)) {
            this.result_count[status] += 1;
        }

        var name_node = row.appendChild(document.createElement("td"));
        name_node.appendChild(this.test_name_node(test));

        var status_node = row.appendChild(document.createElement("td"));
        status_node.textContent = test_status;
        status_node.className = test_status;

        var message_node = row.appendChild(document.createElement("td"));
        message_node.textContent = message || "";

        var subtests_node = row.appendChild(document.createElement("td"));
        if (subtests_count) {
            subtests_node.textContent = subtest_pass_count + "/" + subtests_count;
        } else {
            if (status == "PASS") {
                subtests_node.textContent = "1/1";
            } else {
                subtests_node.textContent = "0/1";
            }
        }

        var status_arr = ["PASS", "FAIL", "ERROR", "TIMEOUT", "NOTRUN"];
        for (var i = 0; i < status_arr.length; i++) {
            this.elem.querySelector("td." + status_arr[i]).textContent = this.result_count[status_arr[i]];
        }

        this.results_table.tBodies[0].appendChild(row);
        this.update_meter(this.runner.progress(), this.runner.results.count(), this.runner.test_count());
    },

    on_done: function() {
        this.meter.setAttribute("aria-valuenow", this.meter.getAttribute("aria-valuemax"));
        this.meter.style.width = "100%";
        if (this.runner.stop_flag) {
            this.meter.textContent = "Stopped";
            this.meter.classList.add("stopped");
        } else {
            this.meter.textContent = "Done!";
        }
        this.meter.classList.remove("progress-striped", "active");
        this.runner.test_div.textContent = "";
        //add the json serialization of the results
        var a = this.elem.querySelector(".jsonResults");
        var json = this.runner.results.to_json();

        if (document.getElementById("dumpit").checked) {
            this.json_results_area = document.createElement("textarea");
            this.json_results_area.style.width = "100%";
            this.json_results_area.setAttribute("rows", "50");
            this.elem.appendChild(this.json_results_area);
            this.json_results_area.textContent = json;
        }
        var blob = new Blob([json], { type: "application/json" });
        a.href = window.URL.createObjectURL(blob);
        a.download = "runner-results.json";
        a.textContent = "Download JSON results";
        if (!a.getAttribute("download")) a.textContent += " (right-click and save as to download)";
        a.style.display = "inline";
    },

    test_name_node: function(test) {
        if (!test.hasOwnProperty("ref_url")) {
            return this.link(test.url);
        } else {
            var wrapper = document.createElement("span");
            wrapper.appendChild(this.link(test.url));
            wrapper.appendChild(document.createTextNode(" " + test.ref_type + " "));
            wrapper.appendChild(this.link(test.ref_url));
            return wrapper;
        }
    },

    link: function(href) {
        var link = document.createElement("a");
        link.href = this.runner.server + href;
        link.textContent = href;
        return link;
    },

    update_meter: function(progress, count, total) {
        this.meter.setAttribute("aria-valuenow", count);
        this.meter.setAttribute("aria-valuemax", total);
        this.meter.textContent = this.meter.style.width = (progress * 100).toFixed(1) + "%";
    }

};

function ManualUI(elem, runner) {
    this.elem = elem;
    this.runner = runner;
    this.pass_button = this.elem.querySelector("button.pass");
    this.fail_button = this.elem.querySelector("button.fail");
    this.ref_buttons = this.elem.querySelector(".reftestUI");
    this.ref_type = this.ref_buttons.querySelector(".refType");
    this.test_button = this.ref_buttons.querySelector("button.test");
    this.ref_button = this.ref_buttons.querySelector("button.ref");

    this.hide();

    this.runner.test_start_callbacks.push(this.on_test_start.bind(this));
    this.runner.test_pause_callbacks.push(this.hide.bind(this));
    this.runner.done_callbacks.push(this.on_done.bind(this));

    this.pass_button.onclick = function() {
        this.disable_buttons();
        this.runner.on_result("PASS", "", []);
    }.bind(this);

    this.fail_button.onclick = function() {
        this.disable_buttons();
        this.runner.on_result("FAIL", "", []);
    }.bind(this);
}

ManualUI.prototype = {
    show: function() {
        this.elem.style.display = "block";
        setTimeout(this.enable_buttons.bind(this), 200);
    },

    hide: function() {
        this.elem.style.display = "none";
    },

    show_ref: function() {
        this.ref_buttons.style.display = "block";
        this.test_button.onclick = function() {
            this.runner.load(this.runner.current_test.url);
        }.bind(this);
        this.ref_button.onclick = function() {
            this.runner.load(this.runner.current_test.ref_url);
        }.bind(this);
    },

    hide_ref: function() {
        this.ref_buttons.style.display = "none";
    },

    disable_buttons: function() {
        this.pass_button.disabled = true;
        this.fail_button.disabled = true;
    },

    enable_buttons: function() {
        this.pass_button.disabled = false;
        this.fail_button.disabled = false;
    },

    on_test_start: function(test) {
        if (test.type == "manual" || test.type == "reftest") {
            this.show();
        } else {
            this.hide();
        }
        if (test.type == "reftest") {
            this.show_ref();
            this.ref_type.textContent = test.ref_type === "==" ? "equal" : "unequal";
        } else {
            this.hide_ref();
        }
    },

    on_done: function() {
        this.hide();
    }
};

function TestControl(elem, runner) {
    this.elem = elem;
    this.path_input = this.elem.querySelector(".path");
    this.pause_button = this.elem.querySelector("button.togglePause");
    this.start_button = this.elem.querySelector("button.toggleStart");
    this.type_checkboxes = Array.prototype.slice.call(
        this.elem.querySelectorAll("input[type=checkbox].test-type"));
    this.type_checkboxes.forEach(function(elem) {
        elem.addEventListener("click", function() {
            this.start_button.disabled = this.get_test_types().length < 1;
        }.bind(this),
        false);
    }.bind(this));
    this.timeout_input = this.elem.querySelector(".timeout_multiplier");
    this.render_checkbox = this.elem.querySelector(".render");
    this.runner = runner;
    this.runner.done_callbacks.push(this.on_done.bind(this));
    this.set_start();
}

TestControl.prototype = {
    set_start: function() {
        this.start_button.disabled = this.get_test_types().length < 1;
        this.pause_button.disabled = true;
        this.start_button.textContent = "Start";
        this.path_input.disabled = false;
        this.type_checkboxes.forEach(function(elem) {
            elem.disabled = false;
        });
        this.start_button.onclick = function() {
            var path = this.get_path();
            var test_types = this.get_test_types();
            var settings = this.get_testharness_settings();
            this.runner.start(path, test_types, settings);
            this.set_stop();
            this.set_pause();
        }.bind(this);
    },

    set_stop: function() {
        clearTimeout(this.runner.timeout);
        this.pause_button.disabled = false;
        this.start_button.textContent = "Stop";
        this.path_input.disabled = true;
        this.type_checkboxes.forEach(function(elem) {
            elem.disabled = true;
        });
        this.start_button.onclick = function() {
            this.runner.stop_flag = true;
            this.runner.done();
        }.bind(this);
    },

    set_pause: function() {
        this.pause_button.textContent = "Pause";
        this.pause_button.onclick = function() {
            this.runner.pause();
            this.set_resume();
        }.bind(this);
    },

    set_resume: function() {
        this.pause_button.textContent = "Resume";
        this.pause_button.onclick = function() {
            this.runner.unpause();
            this.set_pause();
        }.bind(this);

    },

    get_path: function() {
        return this.path_input.value;
    },

    get_test_types: function() {
        return this.type_checkboxes.filter(function(elem) {
            return elem.checked;
        }).map(function(elem) {
            return elem.value;
        });
    },

    get_testharness_settings: function() {
        return {timeout_multiplier: parseFloat(this.timeout_input.value),
                output: this.render_checkbox.checked};
    },

    on_done: function() {
        this.set_pause();
        this.set_start();
    }
};

function Results(runner) {
    this.test_results = null;
    this.runner = runner;

    this.runner.start_callbacks.push(this.on_start.bind(this));
}

Results.prototype = {
    on_start: function() {
        this.test_results = [];
    },

    set: function(test, status, message, subtests) {
        this.test_results.push({"test":test,
                                "subtests":subtests,
                                "status":status,
                                "message":message});
    },

    count: function() {
        return this.test_results.length;
    },

    to_json: function() {
        var data = {
            "results": this.test_results.map(function(result) {
                var rv = {"test":(result.test.hasOwnProperty("ref_url") ?
                                  [result.test.url, result.test.ref_type, result.test.ref_url] :
                                  result.test.url),
                          "subtests":result.subtests,
                          "status":result.status,
                          "message":result.message};
                return rv;
            })
        };
        return JSON.stringify(data, null, 2);
    }
};

function Runner(manifest_path) {
    this.server = location.protocol + "//" + location.host;
    this.manifest = new Manifest(manifest_path);
    this.path = null;
    this.test_types = null;
    this.manifest_iterator = null;

    this.test_window = null;
    this.test_div = document.getElementById('test_url');
    this.current_test = null;
    this.timeout = null;
    this.num_tests = null;
    this.pause_flag = false;
    this.stop_flag = false;
    this.done_flag = false;

    this.manifest_wait_callbacks = [];
    this.start_callbacks = [];
    this.test_start_callbacks = [];
    this.test_pause_callbacks = [];
    this.result_callbacks = [];
    this.done_callbacks = [];

    this.results = new Results(this);

    this.start_after_manifest_load = false;
    this.manifest.load(this.manifest_loaded.bind(this));
}

Runner.prototype = {
    test_timeout: 20000, //ms

    currentTest: function() {
        return this.manifest[this.mTestCount];
    },

    open_test_window: function() {
        this.test_window = window.open("about:blank", 800, 600);
    },

    manifest_loaded: function() {
        if (this.start_after_manifest_load) {
            this.do_start();
        }
    },

    start: function(path, test_types, testharness_settings) {
        this.pause_flag = false;
        this.stop_flag = false;
        this.done_flag = false;
        this.path = path;
        this.test_types = test_types;
        window.testharness_properties = testharness_settings;
        this.manifest_iterator = new ManifestIterator(this.manifest, this.path, this.test_types);
        this.num_tests = null;

        if (this.manifest.data === null) {
            this.wait_for_manifest();
        } else {
            this.do_start();
        }
    },

    wait_for_manifest: function() {
        this.start_after_manifest_load = true;
        this.manifest_wait_callbacks.forEach(function(callback) {
            callback();
        });
    },

    do_start: function() {
        if (this.manifest_iterator.count() > 0) {
            this.open_test_window();
            this.start_callbacks.forEach(function(callback) {
                callback();
            });
            this.run_next_test();
        } else {
            var tests = "tests";
            if (this.test_types.length < 3) {
                tests = this.test_types.join(" tests or ") + " tests";
            }
            var message = "No " + tests + " found in this path."
            document.querySelector(".path").setCustomValidity(message);
            this.done();
        }
    },

    pause: function() {
        this.pause_flag = true;
        this.test_pause_callbacks.forEach(function(callback) {
            callback(this.current_test);
        }.bind(this));
    },

    unpause: function() {
        this.pause_flag = false;
        this.run_next_test();
    },

    on_result: function(status, message, subtests) {
        clearTimeout(this.timeout);
        this.results.set(this.current_test, status, message, subtests);
        this.result_callbacks.forEach(function(callback) {
            callback(this.current_test, status, message, subtests);
        }.bind(this));
        this.run_next_test();
    },

    on_timeout: function() {
        this.on_result("TIMEOUT", "", []);
    },

    done: function() {
        this.done_flag = true;
        if (this.test_window) {
            this.test_window.close();
        }
        this.done_callbacks.forEach(function(callback) {
            callback();
        });
    },

    run_next_test: function() {
        if (this.pause_flag) {
            return;
        }
        var next_test = this.manifest_iterator.next();
        if (next_test === null||this.done_flag) {
            this.done();
            return;
        }

        this.current_test = next_test;

        if (next_test.type === "testharness") {
            this.timeout = setTimeout(this.on_timeout.bind(this),
                                      this.test_timeout * window.testharness_properties.timeout_multiplier);
        }
        this.test_div.textContent = this.current_test.url;
        this.load(this.current_test.url);

        this.test_start_callbacks.forEach(function(callback) {
            callback(this.current_test);
        }.bind(this));
    },

    load: function(path) {
        if (this.test_window.location === null) {
            this.open_test_window();
        }
        this.test_window.location.href = this.server + path;
    },

    progress: function() {
        return this.results.count() / this.test_count();
    },

    test_count: function() {
        if (this.num_tests === null) {
            this.num_tests = this.manifest_iterator.count();
        }
        return this.num_tests;
    }

};


function parseOptions() {
    var options = {
        test_types: ["testharness", "reftest", "manual"]
    };

    var optionstrings = location.search.substring(1).split("&");
    for (var i = 0, il = optionstrings.length; i < il; ++i) {
        var opt = optionstrings[i];
        //TODO: fix this for complex-valued options
        options[opt.substring(0, opt.indexOf("="))] =
            opt.substring(opt.indexOf("=") + 1);
    }
    return options;
}

function setup() {
    var options = parseOptions();

    if (options.path) {
        document.getElementById('path').value = options.path;
    }

    runner = new Runner("/MANIFEST.json", options);
    var test_control = new TestControl(document.getElementById("testControl"), runner);
    new ManualUI(document.getElementById("manualUI"), runner);
    new VisualOutput(document.getElementById("output"), runner);

    if (options.autorun === "1") {
        runner.start(test_control.get_path(),
                     test_control.get_test_types(),
                     test_control.get_testharness_settings());
        return;
    }
}

window.completion_callback = function(tests, status) {
    var harness_status_map = {0:"OK", 1:"ERROR", 2:"TIMEOUT", 3:"NOTRUN"};
    var subtest_status_map = {0:"PASS", 1:"FAIL", 2:"TIMEOUT", 3:"NOTRUN"};

    // this ugly hack is because IE really insists on holding on to the objects it creates in
    // other windows, and on losing track of them when the window gets closed
    var subtest_results = JSON.parse(JSON.stringify(
        tests.map(function (test) {
            return {name: test.name,
                    status: subtest_status_map[test.status],
                    message: test.message};
        })
    ));

    runner.on_result(harness_status_map[status.status],
                     status.message,
                     subtest_results);
};

window.addEventListener("DOMContentLoaded", setup, false);
})();
