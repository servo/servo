"""Methods for the report-event-attribution and report-aggregate-attribution endpoints"""
import json
from typing import List, Optional, Tuple
import urllib.parse

from wptserve.request import Request
from wptserve.stash import Stash
from wptserve.utils import isomorphic_decode, isomorphic_encode

# Key used to access the reports in the stash.
REPORTS = "4691a2d7fca5430fb0f33b1bd8a9d388"
REDIRECT = "9250f93f-2c05-4aae-83b9-2817b0e18b4e"

CLEAR_STASH = isomorphic_encode("clear_stash")
CONFIG_REDIRECT = isomorphic_encode("redirect_to")

Header = Tuple[str, str]
Status = Tuple[int, str]
Response = Tuple[Status, List[Header], str]

def decode_headers(headers: dict) -> dict:
  """Decodes the headers from wptserve.

  wptserve headers are encoded like
  {
    encoded(key): [encoded(value1), encoded(value2),...]
  }
  This method decodes the above using the wptserve.utils.isomorphic_decode
  method
  """
  return {
      isomorphic_decode(key): [isomorphic_decode(el) for el in value
                              ] for key, value in headers.items()
  }

def get_request_origin(request: Request) -> str:
  return "%s://%s" % (request.url_parts.scheme,
                      request.url_parts.netloc)

def configure_redirect(request, origin) -> None:
  with request.server.stash.lock:
      request.server.stash.put(REDIRECT, origin)
      return None

def get_report_redirect_url(request):
  with request.server.stash.lock:
      origin = request.server.stash.take(REDIRECT)
      if origin is None:
         return None
      origin_parts = urllib.parse.urlsplit(origin)
      parts = request.url_parts
      new_parts = origin_parts._replace(path=bytes(parts.path, 'utf-8'))
      return urllib.parse.urlunsplit(new_parts)

def handle_post_report(request: Request, headers: List[Header]) -> Response:
  """Handles POST request for reports.

  Retrieves the report from the request body and stores the report in the
  stash. If clear_stash is specified in the query params, clears the stash.
  """
  if request.GET.get(CLEAR_STASH):
    clear_stash(request.server.stash)
    return (200, "OK"), headers, json.dumps({
        "code": 200,
        "message": "Stash successfully cleared.",
    })

  redirect_origin = request.GET.get(CONFIG_REDIRECT)
  if redirect_origin:
    configure_redirect(request, redirect_origin)
    return (200, "OK"), headers, json.dumps({
        "code": 200,
        "message": "Redirect successfully configured.",
    })

  redirect_url = get_report_redirect_url(request)
  if redirect_url is not None:
    headers.append(("Location", redirect_url))
    return (308, "Permanent Redirect"), headers, json.dumps({
        "code": 308
    })

  store_report(
      request.server.stash, get_request_origin(request), {
          "body": request.body.decode("utf-8"),
          "headers": decode_headers(request.headers)
      })
  return (201, "OK"), headers, json.dumps({
      "code": 201,
      "message": "Report successfully stored."
  })


def handle_get_reports(request: Request, headers: List[Header]) -> Response:
  """Handles GET request for reports.

  Retrieves and returns all reports from the stash.
  """
  reports = take_reports(request.server.stash, get_request_origin(request))
  headers.append(("Access-Control-Allow-Origin", "*"))
  return (200, "OK"), headers, json.dumps({
      "code": 200,
      "reports": reports,
  })


def store_report(stash: Stash, origin: str, report: str) -> None:
  """Stores the report in the stash. Report here is a JSON."""
  with stash.lock:
    reports_dict = stash.take(REPORTS)
    if not reports_dict:
      reports_dict = {}
    reports = reports_dict.get(origin, [])
    reports.append(report)
    reports_dict[origin] = reports
    stash.put(REPORTS, reports_dict)
  return None

def clear_stash(stash: Stash) -> None:
  "Clears the stash."
  stash.take(REPORTS)
  stash.take(REDIRECT)
  return None

def take_reports(stash: Stash, origin: str) -> List[str]:
  """Takes all the reports from the stash and returns them."""
  with stash.lock:
    reports_dict = stash.take(REPORTS)
    if not reports_dict:
      reports_dict = {}

    reports = reports_dict.pop(origin, [])
    stash.put(REPORTS, reports_dict)
  return reports


def handle_reports(request: Request) -> Response:
  """Handles request to get or store reports."""
  headers = [("Content-Type", "application/json")]
  if request.method == "POST":
    return handle_post_report(request, headers)
  if request.method == "GET":
    return handle_get_reports(request, headers)
  return (405, "Method Not Allowed"), headers, json.dumps({
      "code": 405,
      "message": "Only GET or POST methods are supported."
  })
