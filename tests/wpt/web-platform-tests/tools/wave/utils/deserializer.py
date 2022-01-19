from ..data.session import Session, UNKNOWN


def deserialize_sessions(session_dicts):
    sessions = []
    for session_dict in session_dicts:
        session = deserialize_session(session_dict)
        sessions.append(session)
    return sessions


def deserialize_session(session_dict):
    token = ""
    if "token" in session_dict:
        token = session_dict["token"]
    tests = {"include": [], "exclude": []}
    if "tests" in session_dict:
        tests = session_dict["tests"]
    if "path" in session_dict:
        test_paths = session_dict["path"].split(", ")
        tests["include"] = tests["include"] + test_paths
    types = []
    if "types" in session_dict:
        types = session_dict["types"]
    user_agent = ""
    if "user_agent" in session_dict:
        user_agent = session_dict["user_agent"]
    labels = []
    if "labels" in session_dict:
        labels = session_dict["labels"]
    timeouts = {}
    if "timeouts" in session_dict:
        timeouts = session_dict["timeouts"]
    pending_tests = None
    if "pending_tests" in session_dict:
        pending_tests = session_dict["pending_tests"]
    running_tests = None
    if "running_tests" in session_dict:
        running_tests = session_dict["running_tests"]
    status = UNKNOWN
    if "status" in session_dict:
        status = session_dict["status"]
    test_state = None
    if "test_state" in session_dict:
        test_state = session_dict["test_state"]
    last_completed_test = None
    if "last_completed_test" in session_dict:
        last_completed_test = session_dict["last_completed_test"]
    date_started = None
    if "date_started" in session_dict:
        date_started = session_dict["date_started"]
    date_finished = None
    if "date_finished" in session_dict:
        date_finished = session_dict["date_finished"]
    is_public = False
    if "is_public" in session_dict:
        is_public = session_dict["is_public"]
    reference_tokens = []
    if "reference_tokens" in session_dict:
        reference_tokens = session_dict["reference_tokens"]
    browser = None
    if "browser" in session_dict:
        browser = session_dict["browser"]
    webhook_urls = []
    if "webhook_urls" in session_dict:
        webhook_urls = session_dict["webhook_urls"]
    expiration_date = None
    if "expiration_date" in session_dict:
        expiration_date = session_dict["expiration_date"]
    malfunctioning_tests = []
    if "malfunctioning_tests" in session_dict:
        malfunctioning_tests = session_dict["malfunctioning_tests"]

    return Session(
        token=token,
        tests=tests,
        types=types,
        user_agent=user_agent,
        labels=labels,
        timeouts=timeouts,
        pending_tests=pending_tests,
        running_tests=running_tests,
        status=status,
        test_state=test_state,
        last_completed_test=last_completed_test,
        date_started=date_started,
        date_finished=date_finished,
        is_public=is_public,
        reference_tokens=reference_tokens,
        browser=browser,
        webhook_urls=webhook_urls,
        expiration_date=expiration_date,
        malfunctioning_tests=malfunctioning_tests
    )
