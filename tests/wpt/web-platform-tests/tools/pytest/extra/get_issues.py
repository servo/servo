import json
import py
import textwrap

issues_url = "http://bitbucket.org/api/1.0/repositories/pytest-dev/pytest/issues"

import requests

def get_issues():
    chunksize = 50
    start = 0
    issues = []
    while 1:
        post_data = {"accountname": "pytest-dev",
                     "repo_slug": "pytest",
                     "start": start,
                     "limit": chunksize}
        print ("getting from", start)
        r = requests.get(issues_url, params=post_data)
        data = r.json()
        issues.extend(data["issues"])
        if start + chunksize >= data["count"]:
            return issues
        start += chunksize

kind2num = "bug enhancement task proposal".split()

status2num = "new open resolved duplicate invalid wontfix".split()

def main(args):
    cachefile = py.path.local(args.cache)
    if not cachefile.exists() or args.refresh:
        issues = get_issues()
        cachefile.write(json.dumps(issues))
    else:
        issues = json.loads(cachefile.read())

    open_issues = [x for x in issues
                    if x["status"] in ("new", "open")]

    def kind_and_id(x):
        kind = x["metadata"]["kind"]
        return kind2num.index(kind), len(issues)-int(x["local_id"])
    open_issues.sort(key=kind_and_id)
    report(open_issues)

def report(issues):
    for issue in issues:
        metadata = issue["metadata"]
        priority = issue["priority"]
        title = issue["title"]
        content = issue["content"]
        kind = metadata["kind"]
        status = issue["status"]
        id = issue["local_id"]
        link = "https://bitbucket.org/pytest-dev/pytest/issue/%s/" % id
        print("----")
        print(status, kind, link)
        print(title)
        #print()
        #lines = content.split("\n")
        #print ("\n".join(lines[:3]))
        #if len(lines) > 3 or len(content) > 240:
        #    print ("...")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser("process bitbucket issues")
    parser.add_argument("--refresh", action="store_true",
                        help="invalidate cache, refresh issues")
    parser.add_argument("--cache", action="store", default="issues.json",
                        help="cache file")
    args = parser.parse_args()
    main(args)
