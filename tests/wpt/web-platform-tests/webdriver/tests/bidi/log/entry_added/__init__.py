def assert_base_entry(entry,
                      level=None,
                      text=None,
                      time_start=None,
                      time_end=None,
                      stacktrace=None):
    assert "level" in entry
    assert isinstance(entry["level"], str)
    if level is not None:
        assert entry["level"] == level

    assert "text" in entry
    assert isinstance(entry["text"], str)
    if text is not None:
        assert entry["text"] == text

    assert "timestamp" in entry
    assert isinstance(entry["timestamp"], int)
    if time_start is not None:
        assert entry["timestamp"] >= time_start
    if time_end is not None:
        assert entry["timestamp"] <= time_end

    if stacktrace is not None:
        assert "stackTrace" in entry
        assert isinstance(entry["stackTrace"], object)
        assert "callFrames" in entry["stackTrace"]

        call_frames = entry["stackTrace"]["callFrames"]
        assert isinstance(call_frames, list)
        assert len(call_frames) == len(stacktrace)
        for index in range(0, len(call_frames)):
            assert call_frames[index] == stacktrace[index]


def assert_console_entry(entry,
                         method=None,
                         level=None,
                         text=None,
                         args=None,
                         time_start=None,
                         time_end=None,
                         realm=None,
                         stacktrace=None):
    assert_base_entry(entry, level, text, time_start, time_end, stacktrace)

    assert "type" in entry
    assert isinstance(entry["type"], str)
    assert entry["type"] == "console"

    assert "method" in entry
    assert isinstance(entry["method"], str)
    if method is not None:
        assert entry["method"] == method

    assert "args" in entry
    assert isinstance(entry["args"], list)
    if args is not None:
        assert entry["args"] == args

    if realm is not None:
        assert "realm" in entry
        assert isinstance(entry["realm"], str)


def assert_javascript_entry(entry,
                            level=None,
                            text=None,
                            time_start=None,
                            time_end=None,
                            stacktrace=None):
    assert_base_entry(entry, level, text, time_start, time_end, stacktrace)

    assert "type" in entry
    assert isinstance(entry["type"], str)
    assert entry["type"] == "javascript"


def create_console_api_message(current_session, inline, text):
    current_session.execute_script(f"console.log('{text}')")
    return text


def create_javascript_error(current_session, inline, error_message="foo"):
    return current_session.execute_script(f"""
        const script = document.createElement("script");
        script.append(document.createTextNode(`(() => {{throw new Error('{error_message}')}})()`));
        document.body.append(script);
        const err = new Error('{error_message}'); return err.toString()
    """)


def create_log(current_session, inline, log_type, text="foo"):
    if log_type == "console_api_log":
        return create_console_api_message(current_session, inline, text)
    if log_type == "javascript_error":
        return create_javascript_error(current_session, inline, text)
