This docker images is used for testing Chrome, Firefox, WebKitGTK and running
other tasks on Taskcluster. When any of the files in this directory change, the
images must be updated as well. Doing this requires you be part of the
'webplatformtests' organization on Docker Hub; ping @foolip or @jpchase
if you are not a member.

The tag for a new docker image is of the form
`webplatformtests/wpt:{current-version + 0.01}`

To update the docker image:

* Update the following Taskcluster configurations to use the new image:
 - `.taskcluster.yml` (the decision task)
 - `tools/ci/tc/tasks/test.yml` (all the other tasks)

* Run `wpt docker-push`
