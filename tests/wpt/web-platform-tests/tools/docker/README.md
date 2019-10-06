This docker images is used for testing Chrome, Firefox and running other tasks
on Taskcluster. When any of the files in this directory change, the images must
be updated as well. To do this, assuming you have docker installed:

In this directory, run
```sh
docker build -t <tag> .
docker push <tag>
```

Then update the `image` specified in the project's .taskcluster.yml file.

To update the image used for WebKitGTK:
```sh
docker build -f Dockerfile.webkitgtk -t <tag> .
docker push <tag>
```
