from typing import List, Tuple, Union

from fledge.tentative.resources import fledge_http_server_util
from wptserve.request import Request
from wptserve.stash import Stash
from wptserve.utils import isomorphic_decode, isomorphic_encode

# Arbitrary key used to access the reports in the stash.
REPORTS_KEY = "9d285691-4386-45ad-9a79-d2ec29557cde"

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
               request.body)
  return 200, [], ""

def handle_get_request(request: Request) -> Response:
  """Handles GET request for reports.

  Retrieves and returns all reports from the stash.
  """
  headers = [("Content-Type", "text/plain")]
  reports = take_reports(request.server.stash, get_request_origin(request))
  headers.append(("Access-Control-Allow-Origin", "*"))
  return 200, headers, reports

def store_report(stash: Stash, origin: str, report: str) -> None:
  """Stores the report in the stash."""
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

  return (
      (405, "Method Not Allowed"),
      [("Content-Type", "text/plain")],
      "Only GET or POST methods are supported.",
  )
