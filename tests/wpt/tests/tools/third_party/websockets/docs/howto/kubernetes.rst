Deploy to Kubernetes
====================

This guide describes how to deploy a websockets server to Kubernetes_. It
assumes familiarity with Docker and Kubernetes.

We're going to deploy a simple app to a local Kubernetes cluster and to ensure
that it scales as expected.

In a more realistic context, you would follow your organization's practices
for deploying to Kubernetes, but you would apply the same principles as far as
websockets is concerned.

.. _Kubernetes: https://kubernetes.io/

.. _containerize-application:

Containerize application
------------------------

Here's the app we're going to deploy. Save it in a file called
``app.py``:

.. literalinclude:: ../../example/deployment/kubernetes/app.py

This is an echo server with one twist: every message blocks the server for
100ms, which creates artificial starvation of CPU time. This makes it easier
to saturate the server for load testing.

The app exposes a health check on ``/healthz``. It also provides two other
endpoints for testing purposes: ``/inemuri`` will make the app unresponsive
for 10 seconds and ``/seppuku`` will terminate it.

The quest for the perfect Python container image is out of scope of this
guide, so we'll go for the simplest possible configuration instead:

.. literalinclude:: ../../example/deployment/kubernetes/Dockerfile

After saving this ``Dockerfile``, build the image:

.. code-block:: console

    $ docker build -t websockets-test:1.0 .

Test your image by running:

.. code-block:: console

    $ docker run --name run-websockets-test --publish 32080:80 --rm \
        websockets-test:1.0

Then, in another shell, in a virtualenv where websockets is installed, connect
to the app and check that it echoes anything you send:

.. code-block:: console

    $ python -m websockets ws://localhost:32080/
    Connected to ws://localhost:32080/.
    > Hey there!
    < Hey there!
    >

Now, in yet another shell, stop the app with:

.. code-block:: console

    $ docker kill -s TERM run-websockets-test

Going to the shell where you connected to the app, you can confirm that it
shut down gracefully:

.. code-block:: console

    $ python -m websockets ws://localhost:32080/
    Connected to ws://localhost:32080/.
    > Hey there!
    < Hey there!
    Connection closed: 1001 (going away).

If it didn't, you'd get code 1006 (abnormal closure).

Deploy application
------------------

Configuring Kubernetes is even further beyond the scope of this guide, so
we'll use a basic configuration for testing, with just one Service_ and one
Deployment_:

.. literalinclude:: ../../example/deployment/kubernetes/deployment.yaml

For local testing, a service of type NodePort_ is good enough. For deploying
to production, you would configure an Ingress_.

.. _Service: https://kubernetes.io/docs/concepts/services-networking/service/
.. _Deployment: https://kubernetes.io/docs/concepts/workloads/controllers/deployment/
.. _NodePort: https://kubernetes.io/docs/concepts/services-networking/service/#nodeport
.. _Ingress: https://kubernetes.io/docs/concepts/services-networking/ingress/

After saving this to a file called ``deployment.yaml``, you can deploy:

.. code-block:: console

    $ kubectl apply -f deployment.yaml
    service/websockets-test created
    deployment.apps/websockets-test created

Now you have a deployment with one pod running:

.. code-block:: console

    $ kubectl get deployment websockets-test
    NAME              READY   UP-TO-DATE   AVAILABLE   AGE
    websockets-test   1/1     1            1           10s
    $ kubectl get pods -l app=websockets-test
    NAME                               READY   STATUS    RESTARTS   AGE
    websockets-test-86b48f4bb7-nltfh   1/1     Running   0          10s

You can connect to the service — press Ctrl-D to exit:

.. code-block:: console

    $ python -m websockets ws://localhost:32080/
    Connected to ws://localhost:32080/.
    Connection closed: 1000 (OK).

Validate deployment
-------------------

First, let's ensure the liveness probe works by making the app unresponsive:

.. code-block:: console

    $ curl http://localhost:32080/inemuri
    Sleeping for 10s

Since we have only one pod, we know that this pod will go to sleep.

The liveness probe is configured to run every second. By default, liveness
probes time out after one second and have a threshold of three failures.
Therefore Kubernetes should restart the pod after at most 5 seconds.

Indeed, after a few seconds, the pod reports a restart:

.. code-block:: console

    $ kubectl get pods -l app=websockets-test
    NAME                               READY   STATUS    RESTARTS   AGE
    websockets-test-86b48f4bb7-nltfh   1/1     Running   1          42s

Next, let's take it one step further and crash the app:

.. code-block:: console

    $ curl http://localhost:32080/seppuku
    Terminating

The pod reports a second restart:

.. code-block:: console

    $ kubectl get pods -l app=websockets-test
    NAME                               READY   STATUS    RESTARTS   AGE
    websockets-test-86b48f4bb7-nltfh   1/1     Running   2          72s

All good — Kubernetes delivers on its promise to keep our app alive!

Scale deployment
----------------

Of course, Kubernetes is for scaling. Let's scale — modestly — to 10 pods:

.. code-block:: console

    $ kubectl scale deployment.apps/websockets-test --replicas=10
    deployment.apps/websockets-test scaled

After a few seconds, we have 10 pods running:

.. code-block:: console

    $ kubectl get deployment websockets-test
    NAME              READY   UP-TO-DATE   AVAILABLE   AGE
    websockets-test   10/10   10           10          10m

Now let's generate load. We'll use this script:

.. literalinclude:: ../../example/deployment/kubernetes/benchmark.py

We'll connect 500 clients in parallel, meaning 50 clients per pod, and have
each client send 6 messages. Since the app blocks for 100ms before responding,
if connections are perfectly distributed, we expect a total run time slightly
over 50 * 6 * 0.1 = 30 seconds.

Let's try it:

.. code-block:: console

    $ ulimit -n 512
    $ time python benchmark.py 500 6
    python benchmark.py 500 6  2.40s user 0.51s system 7% cpu 36.471 total

A total runtime of 36 seconds is in the right ballpark. Repeating this
experiment with other parameters shows roughly consistent results, with the
high variability you'd expect from a quick benchmark without any effort to
stabilize the test setup.

Finally, we can scale back to one pod.

.. code-block:: console

    $ kubectl scale deployment.apps/websockets-test --replicas=1
    deployment.apps/websockets-test scaled
    $ kubectl get deployment websockets-test
    NAME              READY   UP-TO-DATE   AVAILABLE   AGE
    websockets-test   1/1     1            1           15m
