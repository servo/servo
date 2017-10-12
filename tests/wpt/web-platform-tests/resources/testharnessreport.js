var props = {output:%(output)d,
             explicit_timeout: true,
             message_events: ["completion"]};

if (window.opener && "timeout_multiplier" in window.opener) {
    props["timeout_multiplier"] = window.opener.timeout_multiplier;
}

if (window.opener && window.opener.explicit_timeout) {
    props["explicit_timeout"] = window.opener.explicit_timeout;
}

setup(props);
