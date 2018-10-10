# Windows AMIs for Servo on Taskcluster

Unlike Linux tasks on `docker-worker` where each tasks is executed in a container
based on a Docker image provided with the task,
Windows tasks on Taskcluster are typically run by `generic-worker`
where tasks are executed directly in the worker’s environment.
So we may want to install some tools globally on the system, to make them available to tasks.

With the [AWS provisioner], this means building a custom AMI.
We need to boot an instance on a base Windows AMI,
install what we need (including `generic-worker` itself),
then take an image of that instance.
The [`worker_types`] directory in `generic-worker`’s repository
has some scripts that automate this,
in order to make it more reproducible than clicking around.
The trick is that a PowerShell script to run on boot can be provided
when starting a Windows instance on EC2, and of course AWS has an API.

[AWS provisioner]: https://docs.taskcluster.net/docs/reference/integrations/aws-provisioner/references/api
[`worker_types`]: https://github.com/taskcluster/generic-worker/blob/master/worker_types/


## Building and deploying a new image

* Install and configure the [AWS command-line tool].
* Make your changes to `first-boot.ps1` and/or `base-ami.txt`.
* Run `python3 build-ami.py`. Note that it can take many minutes to complete.
* Save the administrator password together with the image ID
  in Servo’s shared 1Password account, in the *Taskcluster Windows AMIs* note.
* In the [worker type definition], edit `ImageId` and `DeploymentId`.

Note that the new worker type definition will only apply to newly-provisionned workers.

`DeploymentId` can be any string. It can for example include the image ID.
Workers check it between tasks (if `checkForNewDeploymentEverySecs` since the last check).
If it has changed, they shut down in order to leave room for new workers with the new definition.

The [EC2 Resources] page has red *Terminate All Instances* button,
but that will make any running task fail.

[AWS command-line tool]: https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html
[worker type definition]: https://tools.taskcluster.net/aws-provisioner/servo-win2016/edit
[EC2 Resources]: https://tools.taskcluster.net/aws-provisioner/servo-win2016/resources


## FIXME: possible improvement

* Have a separate staging worker type to try new AMIs without affecting the production CI
* Automate cleaning up old, unused AMIs and their backing EBS snapshots
* Use multiple AWS regions
* Use the Taskcluster API to automate updating worker type definitions?


## Picking a base AMI

Amazon provides an ovewhelming number of different Windows images,
so it’s hard to find what’s relevant.
Their console might show a paginated view like this:

> ⇤ ← 1 to 50 of 13,914 AMIs → ⇥

Let’s grep through this with the API:

```sh
aws ec2 describe-images --owners amazon --filters 'Name=platform,Values=windows' \
    --query 'Images[*].[ImageId,Name,Description]' --output table > /tmp/images
< /tmp/images less -S
```

It turns out that these images are all based on Windows Server,
but their number is explained by the presence of many (all?) combinations of:

* Multiple OS Version
* Many available locales
* *Full* (a.k.a. *with Desktop Experience*), or *Core*
* *Base* with only the OS, or multiple flavors with tools like SQL Server pre-installed

If we make some choices and filter the list:

```sh
< /tmp/images grep 2016-English-Full-Base | less -S
```

… we get a much more manageable handlful of images with names like
`Windows_Server-2016-English-Full-Base-2018.09.15` or other dates.

Let’s set `base-ami.txt` to `Windows_Server-2016-English-Full-Base-*`,
and have `build-ami.py` pick the most recently-created AMI whose name matches that pattern.