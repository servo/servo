"use strict";

(function() {
    const is_test_context = window.__wptrunner_message_queue !== undefined;
    const pending = new Map();

    let result = null;
    let ctx_cmd_id = 0;
    let testharness_context = null;

    window.addEventListener("message", function(event) {
        const data = event.data;

        if (typeof data !== "object" && data !== null) {
            return;
        }

        if (is_test_context && data.type === "testdriver-command") {
            const command = data.message;
            const ctx_id = command.cmd_id;
            delete command.cmd_id;
            const cmd_id = window.__wptrunner_message_queue.push(command);
            let on_success = (data) => {
                data.type = "testdriver-complete";
                data.cmd_id = ctx_id;
                event.source.postMessage(data, "*");
            };
            let on_failure = (data) => {
                data.type = "testdriver-complete";
                data.cmd_id = ctx_id;
                event.source.postMessage(data, "*");
            };
            pending.set(cmd_id, [on_success, on_failure]);
        } else if (data.type === "testdriver-complete") {
            const cmd_id = data.cmd_id;
            const [on_success, on_failure] = pending.get(cmd_id);
            pending.delete(cmd_id);
            const resolver = data.status === "success" ? on_success : on_failure;
            resolver(data);
            if (is_test_context) {
                window.__wptrunner_process_next_event();
            }
        }
    });

    // Code copied from /common/utils.js
    function rand_int(bits) {
        if (bits < 1 || bits > 53) {
            throw new TypeError();
        } else {
            if (bits >= 1 && bits <= 30) {
                return 0 | ((1 << bits) * Math.random());
            } else {
                var high = (0 | ((1 << (bits - 30)) * Math.random())) * (1 << 30);
                var low = 0 | ((1 << 30) * Math.random());
                return  high + low;
            }
        }
    }

    function to_hex(x, length) {
        var rv = x.toString(16);
        while (rv.length < length) {
            rv = "0" + rv;
        }
        return rv;
    }

    function get_window_id(win) {
        if (win == window && is_test_context) {
            return null;
        }
        if (!win.__wptrunner_id) {
            // generate a uuid
            win.__wptrunner_id = [to_hex(rand_int(32), 8),
                                  to_hex(rand_int(16), 4),
                                  to_hex(0x4000 | rand_int(12), 4),
                                  to_hex(0x8000 | rand_int(14), 4),
                                  to_hex(rand_int(48), 12)].join("-");
        }
        return win.__wptrunner_id;
    }

    const get_context = function(element) {
        if (!element) {
            return null;
        }
        let elementWindow = element.ownerDocument.defaultView;
        if (!elementWindow) {
            throw new Error("Browsing context for element was detached");
        }
        return elementWindow;
    };

    const get_selector = function(element) {
        let selector;

        if (element.id) {
            const id = element.id;

            selector = "#";
            // escape everything, because it's easy to implement
            for (let i = 0, len = id.length; i < len; i++) {
                selector += '\\' + id.charCodeAt(i).toString(16) + ' ';
            }
        } else {
            // push and then reverse to avoid O(n) unshift in the loop
            let segments = [];
            for (let node = element;
                 node.parentElement;
                 node = node.parentElement) {
                let segment = "*|" + node.localName;
                let nth = Array.prototype.indexOf.call(node.parentElement.children, node) + 1;
                segments.push(segment + ":nth-child(" + nth + ")");
            }
            segments.push(":root");
            segments.reverse();

            selector = segments.join(" > ");
        }

        return selector;
    };

    const create_action = function(name, props) {
        let cmd_id;
        const action_msg = {type: "action",
                            action: name,
                            ...props};
        if (action_msg.context) {
          action_msg.context = get_window_id(action_msg.context);
        }
        if (is_test_context) {
            cmd_id = window.__wptrunner_message_queue.push(action_msg);
        } else {
            if (testharness_context === null) {
                throw new Error("Tried to run in a non-testharness window without a call to set_test_context");
            }
            if (action_msg.context === null) {
                action_msg.context = get_window_id(window);
            }
            cmd_id = ctx_cmd_id++;
            action_msg.cmd_id = cmd_id;
            window.test_driver.message_test({type: "testdriver-command",
                                             message: action_msg});
        }
        const pending_promise = new Promise(function(resolve, reject) {
            const on_success = data => {
                result = JSON.parse(data.message).result;
                resolve(result);
            };
            const on_failure = data => {
                reject(`${data.status}: ${data.message}`);
            };
            pending.set(cmd_id, [on_success, on_failure]);
        });
        return pending_promise;
    };

    window.test_driver_internal.in_automation = true;

    window.test_driver_internal.set_test_context = function(context) {
        if (window.__wptrunner_message_queue) {
            throw new Error("Tried to set testharness context in a window containing testharness.js");
        }
        testharness_context = context;
    };

    window.test_driver_internal.click = function(element) {
        const selector = get_selector(element);
        const context = get_context(element);
        return create_action("click", {selector, context});
    };

    window.test_driver_internal.delete_all_cookies = function(context=null) {
        return create_action("delete_all_cookies", {context});
    };

    window.test_driver_internal.minimize_window = function(context=null) {
        return create_action("minimize_window", {context});
    };

    window.test_driver_internal.set_window_rect = function(rect, context=null) {
        return create_action("set_window_rect", {rect, context});
    };

    window.test_driver_internal.send_keys = function(element, keys) {
        const selector = get_selector(element);
        const context = get_context(element);
        return create_action("send_keys", {selector, keys, context});
    };

    window.test_driver_internal.action_sequence = function(actions, context=null) {
        for (let actionSequence of actions) {
            if (actionSequence.type == "pointer") {
                for (let action of actionSequence.actions) {
                    // The origin of each action can only be an element or a string of a value "viewport" or "pointer".
                    if (action.type == "pointerMove" && typeof(action.origin) != 'string') {
                        let action_context = get_context(action.origin);
                        action.origin = {selector: get_selector(action.origin)};
                        if (context !== null && action_context !== context) {
                            throw new Error("Actions must be in a single context");
                        }
                        context = action_context;
                    }
                }
            }
        }
        return create_action("action_sequence", {actions, context});
    };

    window.test_driver_internal.generate_test_report = function(message, context=null) {
        return create_action("generate_test_report", {message, context});
    };

    window.test_driver_internal.set_permission = function(permission_params, context=null) {
        return create_action("set_permission", {permission_params, context});
    };

    window.test_driver_internal.add_virtual_authenticator = function(config, context=null) {
        return create_action("add_virtual_authenticator", {config, context});
    };

    window.test_driver_internal.remove_virtual_authenticator = function(authenticator_id, context=null) {
        return create_action("remove_virtual_authenticator", {authenticator_id, context});
    };

    window.test_driver_internal.add_credential = function(authenticator_id, credential, context=null) {
        return create_action("add_credential", {authenticator_id, credential, context});
    };

    window.test_driver_internal.get_credentials = function(authenticator_id, context=null) {
        return create_action("get_credentials", {authenticator_id, context});
    };

    window.test_driver_internal.remove_credential = function(authenticator_id, credential_id, context=null) {
        return create_action("remove_credential", {authenticator_id, credential_id, context});
    };

    window.test_driver_internal.remove_all_credentials = function(authenticator_id, context=null) {
        return create_action("remove_all_credentials", {authenticator_id, context});
    };

    window.test_driver_internal.set_user_verified = function(authenticator_id, uv, context=null) {
        return create_action("set_user_verified", {authenticator_id, uv, context});
    };

    window.test_driver_internal.set_spc_transaction_mode = function(mode, context = null) {
        return create_action("set_spc_transaction_mode", {mode, context});
    };
})();
