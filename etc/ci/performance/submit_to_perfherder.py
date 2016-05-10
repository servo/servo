import argparse
from functools import partial
import json
import operator
import random
import string
from thclient import (TreeherderClient, TreeherderResultSetCollection,
                      TreeherderJobCollection)


def geometric_mean(iterable):
        iterable = filter(lambda x: x > 0, iterable)
        return (reduce(operator.mul, iterable)) ** (1.0/len(iterable))


def format_testcase_name(name):
    temp = name.replace('http://localhost:8000/page_load_test/', '')
    temp = temp.split('/')[0]
    temp = temp[0:80]
    return temp


def format_perf_data(perf_json):
    suites = []
    measurement = "domComplete"  # Change this to an array when we have more

    def getTimeFromNavStart(timings, measurement):
        return timings[measurement] - timings['navigationStart']

    measurementFromNavStart = partial(getTimeFromNavStart,
                                      measurement=measurement)

    suite = {
        "name": measurement,
        "value": geometric_mean(map(measurementFromNavStart, perf_json)),
        "subtests": []
    }
    for testcase in perf_json:
        if measurementFromNavStart(testcase) < 0:
            value = -1
            print('Error: test case has negative timing. Test timeout?')
        else:
            value = measurementFromNavStart(testcase)

        suite["subtests"].append({
            "name": format_testcase_name(testcase["testcase"]),
            "value": value}
        )

    suites.append(suite)

    return (
        {
            "performance_data": {
                "framework": {"name": "talos"},
                "suites": suites
            }
        }
    )


# TODO: refactor this big function to smaller chunks
def submit(perf_data, revision):

    print("[DEBUG] performance data:")
    print(perf_data)
    # TODO: read the correct guid from test result
    hashlen = len(revision['commit'])
    job_guid = ''.join(
        random.choice(string.letters + string.digits) for i in xrange(hashlen)
    )

    trsc = TreeherderResultSetCollection()

    author = "{} <{}>".format(revision['author']['name'],
                              revision['author']['email'])

    dataset = [
        {
            # The top-most revision in the list of commits for a push.
            'revision': revision['commit'],
            'author': author,
            'push_timestamp': int(revision['author']['timestamp']),
            'type': 'push',
            # a list of revisions associated with the resultset. There should
            # be at least one.
            'revisions': [
                {
                    'comment': revision['subject'],
                    'revision': revision['commit'],
                    'repository': 'servo',
                    'author': author
                }
            ]
        }
    ]

    for data in dataset:

        trs = trsc.get_resultset()

        trs.add_push_timestamp(data['push_timestamp'])
        trs.add_revision(data['revision'])
        trs.add_author(data['author'])
        # trs.add_type(data['type'])

        revisions = []
        for rev in data['revisions']:

            tr = trs.get_revision()

            tr.add_revision(rev['revision'])
            tr.add_author(rev['author'])
            tr.add_comment(rev['comment'])
            tr.add_repository(rev['repository'])

            revisions.append(tr)

        trs.add_revisions(revisions)

        trsc.add(trs)

    dataset = [
        {
            'project': 'servo',
            'revision': revision['commit'],
            'job': {
                'job_guid': job_guid,
                'product_name': 'servo',
                'reason': 'scheduler',
                # TODO:What is `who` for?
                'who': 'Servo',
                'desc': 'Servo Page Load Time Tests',
                'name': 'Servo Page Load Time',
                # The symbol representing the job displayed in
                # treeherder.allizom.org
                'job_symbol': 'PL',

                # The symbol representing the job group in
                # treeherder.allizom.org
                'group_symbol': 'SP',
                'group_name': 'Servo Perf',

                # TODO: get the real timing from the test runner
                'submit_timestamp': revision['author']['timestamp'],
                'start_timestamp':  revision['author']['timestamp'],
                'end_timestamp':  revision['author']['timestamp'],

                'state': 'completed',
                'result': 'success',

                'machine': 'local-machine',
                # TODO: read platform test result
                'build_platform': {
                    'platform': 'linux64',
                    'os_name': 'linux',
                    'architecture': 'x86_64'
                },
                'machine_platform': {
                    'platform': 'linux64',
                    'os_name': 'linux',
                    'architecture': 'x86_64'
                },

                'option_collection': {'opt': True},

                # jobs can belong to different tiers
                # setting the tier here will determine which tier the job
                # belongs to.  However, if a job is set as Tier of 1, but
                # belongs to the Tier 2 profile on the server, it will still
                # be saved as Tier 2.
                'tier': 1,

                # the ``name`` of the log can be the default of "buildbot_text"
                # however, you can use a custom name.  See below.
                # TODO: point this to the log when we have them uploaded
                'log_references': [
                    {
                        'url': 'TBD',
                        'name': 'test log'
                    }
                ],
                # The artifact can contain any kind of structured data
                # associated with a test.
                'artifacts': [
                    {
                        'type': 'json',
                        'name': 'performance_data',
                        # 'job_guid': job_guid,
                        'blob': perf_data
                        # {
                        #    "performance_data": {
                        #        # that is not `talos`?
                        #        "framework": {"name": "talos"},
                        #        "suites": [{
                        #            "name": "performance.timing.domComplete",
                        #            "value": random.choice(range(15,25)),
                        #            "subtests": [
                        #                {"name": "responseEnd", "value": 123},
                        #                {"name": "loadEventEnd", "value": 223}
                        #            ]
                        #        }]
                        #     }
                        # }
                    },
                    {
                        'type': 'json',
                        'name': 'Job Info',
                        # 'job_guid': job_guid,
                        "blob": {
                            "job_details": [
                                {
                                    "url": "https://www.github.com/servo/servo",
                                    "value": "website",
                                    "content_type": "link",
                                    "title": "Source code"
                                }
                            ]
                        }
                    }
                ],
                # List of job guids that were coalesced to this job
                'coalesced': []
            }
        }
    ]

    tjc = TreeherderJobCollection()

    for data in dataset:

        tj = tjc.get_job()

        tj.add_revision(data['revision'])
        tj.add_project(data['project'])
        tj.add_coalesced_guid(data['job']['coalesced'])
        tj.add_job_guid(data['job']['job_guid'])
        tj.add_job_name(data['job']['name'])
        tj.add_job_symbol(data['job']['job_symbol'])
        tj.add_group_name(data['job']['group_name'])
        tj.add_group_symbol(data['job']['group_symbol'])
        tj.add_description(data['job']['desc'])
        tj.add_product_name(data['job']['product_name'])
        tj.add_state(data['job']['state'])
        tj.add_result(data['job']['result'])
        tj.add_reason(data['job']['reason'])
        tj.add_who(data['job']['who'])
        tj.add_tier(data['job']['tier'])
        tj.add_submit_timestamp(data['job']['submit_timestamp'])
        tj.add_start_timestamp(data['job']['start_timestamp'])
        tj.add_end_timestamp(data['job']['end_timestamp'])
        tj.add_machine(data['job']['machine'])

        tj.add_build_info(
            data['job']['build_platform']['os_name'],
            data['job']['build_platform']['platform'],
            data['job']['build_platform']['architecture']
        )

        tj.add_machine_info(
            data['job']['machine_platform']['os_name'],
            data['job']['machine_platform']['platform'],
            data['job']['machine_platform']['architecture']
        )

        tj.add_option_collection(data['job']['option_collection'])

        # for log_reference in data['job']['log_references']:
        #    tj.add_log_reference( 'buildbot_text', log_reference['url'])

        # data['artifact'] is a list of artifacts
        for artifact_data in data['job']['artifacts']:
            tj.add_artifact(
                artifact_data['name'],
                artifact_data['type'],
                artifact_data['blob']
            )
        tjc.add(tj)

    # TODO: extract this read credential code out of this function.
    with open('credential.json', 'rb') as f:
        cred = json.load(f)

    client = TreeherderClient(protocol='https',
                              # host='local.treeherder.mozilla.org',
                              host='treeherder.allizom.org',
                              client_id=cred['client_id'],
                              secret=cred['secret'])

    # data structure validation is automatically performed here, if validation
    # fails a TreeherderClientError is raised
    client.post_collection('servo', trsc)
    client.post_collection('servo', tjc)


def main():
    parser = argparse.ArgumentParser(
        description=("Submit Servo performance data to Perfherder. "
                     "Put your treeherder credentail in credentail.json. "
                     "You can refer to credential.json.example.")
    )
    parser.add_argument("perf_json",
                        help="the output json from runner")
    parser.add_argument("revision_json",
                        help="the json containing the servo revision data")
    args = parser.parse_args()

    with open(args.perf_json, 'rb') as f:
        result_json = json.load(f)

    with open(args.revision_json, 'rb') as f:
        revision = json.load(f)

    perf_data = format_perf_data(result_json)

    submit(perf_data, revision)
    print("Done!")


if __name__ == "__main__":
    main()
