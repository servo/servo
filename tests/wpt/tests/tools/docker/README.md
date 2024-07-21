This docker images is used for testing Chrome, Firefox, WebKitGTK and running
other tasks on Taskcluster. When any of the files in this directory change, the
images must be updated as well. Doing this requires you to have write
permissions to the repository.

The tag for a new docker image is of the form
`ghcr.io/web-platform-tests/wpt:${PREV_VERSION++}`.

To update the docker image:

* Run the workflow
  https://github.com/web-platform-tests/wpt/actions/workflows/docker.yml via the
  GitHub UI.

* Update the following Taskcluster configurations to use the new image:
 - `.taskcluster.yml` (the decision task)
 - `tools/ci/tc/tasks/test.yml` (all the other tasks)