"""Methods for the report-shared-storage and report-protected-audience endpoints (including debug endpoints)"""
import json
from typing import List, Optional, Tuple, Union
import urllib.parse

from wptserve.request import Request
from wptserve.stash import Stash
from wptserve.utils import isomorphic_decode, isomorphic_encode

# Arbitrary key used to access the reports in the stash.
REPORTS_KEY = "9d285691-4386-45ad-9a79-d2ec29557bfe"

CLEAR_STASH_AS_BYTES = isomorphic_encode("clear_stash")

Header = Tuple[str, str]
Status = Union[int, Tuple[int, str]]
Response = Tuple[Status, List[Header], str]

def get_request_origin(request: Request) -> str:
  return "%s://%s" % (request.url_parts.scheme,
                      request.url_parts.netloc)

def handle_post_request(request: Request) -> Response:
  """Handles POST request for reports.

  Retrieves the report from the request body and stores the report in the
  stash. If clear_stash is specified in the query params, clears the stash.
  """
  if request.GET.get(CLEAR_STASH_AS_BYTES):
    clear_stash(request.server.stash)
    return 200, [], "Stash successfully cleared."

  store_report(request.server.stash, get_request_origin(request),
               request.body.decode("utf-8"))
  return 200, [], ""


def handle_get_request(request: Request) -> Response:
  """Handles GET request for reports.

  Retrieves and returns all reports from the stash.
  """
  headers = [("Content-Type", "application/json")]
  reports = take_reports(request.server.stash, get_request_origin(request))
  headers.append(("Access-Control-Allow-Origin", "*"))
  return 200, headers, json.dumps(reports)


def store_report(stash: Stash, origin: str, report: str) -> None:
  """Stores the report in the stash. Report here is a JSON."""
  with stash.lock:
    reports_dict = stash.take(REPORTS_KEY)
    if not reports_dict:
      reports_dict = {}
    reports = reports_dict.get(origin, [])
    reports.append(report)
    reports_dict[origin] = reports
    stash.put(REPORTS_KEY, reports_dict)
  return None

def clear_stash(stash: Stash) -> None:
  "Clears the stash."
  stash.take(REPORTS_KEY)
  return None

def take_reports(stash: Stash, origin: str) -> List[str]:
  """Takes all the reports from the stash and returns them."""
  with stash.lock:
    reports_dict = stash.take(REPORTS_KEY)
    if not reports_dict:
      reports_dict = {}

    reports = reports_dict.pop(origin, [])
    stash.put(REPORTS_KEY, reports_dict)
  return reports


def handle_request(request: Request) -> Response:
  """Handles request to get or store reports."""
  if request.method == "POST":
    return handle_post_request(request)
  if request.method == "GET":
    return handle_get_request(request)

  return (405, "Method Not Allowed"), [("Content-Type", "application/json")], json.dumps({
      "code": 405,
      "message": "Only GET or POST methods are supported."
  })
