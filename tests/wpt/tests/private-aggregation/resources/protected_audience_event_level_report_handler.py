"""Handler to receive message from protected audience worklets, such as
sendReportTo() and forDebuggingOnly.reportAdAuctionWin().

This handler only supports read and write operations from the URL parameters.
"""

import json
from typing import List, Tuple

from wptserve.request import Request
from wptserve.response import Response

Header = Tuple[str, str]
ResponseTuple = Tuple[int, List[Header], str]

def main(request: Request, response: Response) -> ResponseTuple:
    operation = request.GET.first(b"operation").decode('utf-8')
    uuid = request.GET.first(b"uuid").decode('utf-8')
    if operation == "read":
        with request.server.stash.lock:
            stash_reports = request.server.stash.take(key=uuid)
            if stash_reports is None:
                stash_reports = []
            else:
                request.server.stash.put(key=uuid, value=stash_reports)

        return 200, [("Content-Type", "application/json")], json.dumps(stash_reports)
    elif operation == "write":
        report = request.GET.first(b"report").decode('utf-8')

        if report is None:
            return 400, [("Content-Type", "application/json")], json.dumps({'error': 'Missing report.', 'uuid': uuid})

        with request.server.stash.lock:
            stash_reports = request.server.stash.take(key=uuid)
            if stash_reports is None:
                stash_reports = []
            stash_reports.append(report)
            request.server.stash.put(key=uuid, value=stash_reports)

        return 200, [("Content-Type", "application/json")], json.dumps({'msg': 'Recorded report ' + uuid})
    else:
        return 400, [("Content-Type", "application/json")], json.dumps({'error': 'Invalid operation.', 'uuid': uuid})
