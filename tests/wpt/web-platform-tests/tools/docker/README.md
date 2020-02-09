This docker images is used for testing Chrome, Firefox, WebKitGTK and running
other tasks on Taskcluster. When any of the files in this directory change, the
images must be updated as well. To do this, assuming you have docker installed:

In this directory, run
```sh
docker build -t <tag> .
docker push <tag>
```

