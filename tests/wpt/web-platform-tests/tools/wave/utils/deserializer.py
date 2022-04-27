from ..data.session import Session, UNKNOWN
from datetime import datetime
import dateutil.parser
from dateutil.tz import tzutc


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
    test_types = []
    if "types" in session_dict:
        test_types = session_dict["types"]
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
    date_created = None
    if "date_created" in session_dict:
        date_created = session_dict["date_created"]
        date_created = iso_to_millis(date_created)
    date_started = None
    if "date_started" in session_dict:
        date_started = session_dict["date_started"]
        date_started = iso_to_millis(date_started)
    date_finished = None
    if "date_finished" in session_dict:
        date_finished = session_dict["date_finished"]
        date_finished = iso_to_millis(date_finished)
    is_public = False
    if "is_public" in session_dict:
        is_public = session_dict["is_public"]
    reference_tokens = []
    if "reference_tokens" in session_dict:
        reference_tokens = session_dict["reference_tokens"]
    browser = None
    if "browser" in session_dict:
        browser = session_dict["browser"]
    expiration_date = None
    if "expiration_date" in session_dict:
        expiration_date = session_dict["expiration_date"]
        expiration_date = iso_to_millis(expiration_date)
    type = None
    if "type" in session_dict:
        type = session_dict["type"]
    malfunctioning_tests = []
    if "malfunctioning_tests" in session_dict:
        malfunctioning_tests = session_dict["malfunctioning_tests"]

    return Session(
        token=token,
        tests=tests,
        test_types=test_types,
        user_agent=user_agent,
        labels=labels,
        timeouts=timeouts,
        pending_tests=pending_tests,
        running_tests=running_tests,
        status=status,
        test_state=test_state,
        last_completed_test=last_completed_test,
        date_created=date_created,
        date_started=date_started,
        date_finished=date_finished,
        is_public=is_public,
        reference_tokens=reference_tokens,
        browser=browser,
        expiration_date=expiration_date,
        type=type,
        malfunctioning_tests=malfunctioning_tests
    )

def iso_to_millis(iso_string):
    if iso_string is None:
        return None
    try:
        date = dateutil.parser.isoparse(iso_string)
        date = date.replace(tzinfo=tzutc())
        epoch = datetime.utcfromtimestamp(0).replace(tzinfo=tzutc())
        return int((date - epoch).total_seconds() * 1000)
    except Exception:
        return iso_string
