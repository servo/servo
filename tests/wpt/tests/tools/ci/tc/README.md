# Taskgraph Setup

The taskgraph is built from a YAML file. This file has two top-level
properties: `components` and `tasks`. The full list of tasks is
defined by the `tasks` object; each task is an object with a single
property representing the task with the corresponding value an object
representing the task properties. Each task requires the following
top-level properties:

* `provisionerId`: String. Name of Taskcluster provisioner
* `schedulerId`: String. Name of Taskcluster scheduler
* `deadline`: String. Time until the task expires
* `image`: String. Name of docker image to use for task
* `maxRunTime`: Number. Maximum time in seconds for which the task can
  run.
* `artifacts`: Object. List of artifacts and directories to upload; see
  Taskcluster documentation.
* `command`: String. Command to run. This is automatically wrapped in a
  run_tc command
* `options`: Optional Object. Options to pass into run_tc
  - xvfb: Boolean. Enable Xvfb for run
  - oom-killer: Boolean. Enable xvfb for run
  - hosts: Boolean. Update hosts file with wpt hosts before run
  - install-certificates: Boolean. Install wpt certs into OS
    certificate store for run
  - browser: List. List of browser names for run
  - channel: String. Browser channel for run
* `trigger`: Object. Conditions on which to consider task. One or more
  of following properties:
  - branch: List. List of branch names on which to trigger.
  - pull-request: No value. Trigger for pull request actions
* `schedule-if`: Optional Object. Conditions on which task should be
  scheduled given it meets the trigger conditions.
  - `run-job`: List. Job names for which this task should be considered,
    matching the output from `./wpt test-jobs`
* `env`: Optional Object. Environment variables to set when running task.
* `depends-on`: Optional list. List of task names that must be complete
  before the current task is scheduled.
* `description`: String. Task description.
* `name`: Optional String. Name to use for the task overriding the
  property name. This is useful in combination with substitutions
  described below.
* `download-artifacts`: Optional Object. An artifact to download from
  a task that this task depends on. This has the following properties:
  - `task` - Name of the task producing the artifact
  - `glob` - A glob pattern for the filename of the artifact
  - `dest` - A directory reltive to the home directory in which to place
             the artifact
  - `extract` - Optional. A boolean indicating whether an archive artifact
                should be extracted in-place.

## Task Expansions

Using the above syntax it's possble to describe each task
directly. But typically in a taskgraph there are many common
properties between tasks so it's tedious and error prone to repeat
information that's common to multiple tasks. Therefore the taskgraph
format provides several mechanisms to reuse partial task definitions
across multiple tasks.

### Components

The other top-level property in the taskgraph format is
`components`. The value of this property is an object containing named
partial task definitions. Each task definition may contain a property called
`use` which is a list of components to use as the basis for the task
definition. The components list is evaluated in order. If a property
is not previously defined in the output it is added to the output. If
it was previously defined, the value is updated according to the type:
 * Strings and numbers are replaced with a new value
 * Lists are extended with the additional values
 * Objects are updated recursively following the above rules
This means that types must always match between components and the
final value.

For example
```
components:
  example-1:
    list_prop:
      - first
      - second
    object_prop:
      key1: value1
      key2: base_value
  example-2:
    list_prop:
      - third
      - fourth
    object_prop:
      key3:
        - value3-1

tasks:
  - example-task:
      use:
        - example-1
        - example-2
      object_prop:
        key2: value2
        key3:
          - value3-2
```

will evaluate to the following task:

```
example-task:
  list_prop:
    - first
    - second
    - third
    - fourth
  object_prop:
    key1: value1
    key2: value2
    key3:
      - value3-1
      - value3-2
```

Note that components cannot currently define `use` properties of their own.

## Substitutions

Components and tasks can define a property `vars` that holds variables
which are later substituted into the task definition using the syntax
`${vars.property-name}`. For example:

```
components:
  generic-component:
    prop: ${vars.value}

tasks:
  - first:
      use:
        - generic-component
      vars:
        value: value1
  - second:
      use:
        - generic-component
      vars:
        value: value2
```

Results in the following tasks:

```
first:
  prop: value1
second:
  prop: value2
```

## Maps

Instead of defining a task directly, an item in the tasks property may
be an object with a single property `$map`. This object itself has two
child properties; `for` and `do`. The value of `for` is a list of
objects, and the value of `do` is either an object or a list of
objects. For each object in the `for` property, a set of tasks is
created by taking a copy of that object for each task in the `do`
property, updating the object with the properties from the
corresponding `do` object, using the same rules as for components
above, and then processing as for a normal task. `$map` rules can also
be nested.

Note: Although `$map` shares a name with the `$map` used in json-e
(used. in `.taskcluster.yml`), the semantics are different.

For example

```
components: {}
tasks:
  $map:
    for:
      - vars:
          example: value1
      - vars:
          example: value2
    do:
      example-${vars.example}
        prop: ${vars.example}
```

Results in the tasks

```
example-value1:
  prop: value1
example-value2:
  prop: value2
```

Note that in combination with `$map`, variable substitutions are
applied *twice*; once after the `$map` is evaluated and once after the
`use` statements are evaluated.

## Chunks

A common requirements for tasks is that they are "chunked" into N
partial tasks. This is handled specially in the syntax. A top level
property `chunks` can be used to define the number of individual
chunks to create for a specific task. Each chunked task is created
with a `chunks` property set to an object containing an `id` property
containing the one-based index of the chunk an a `total` property
containing the total number of chunks. These can be substituted into
the task definition using the same syntax as for `vars` above
e.g. `${chunks.id}`. Note that because task names must be unique, it's
common to specify a `name` property on the task that will override the
property name e.g.

```
components: {}
tasks:
  - chunked-task:
      chunks:2
      command: "task-run --chunk=${chunks.id} --totalChunks=${chunks.total}"
      name: task-chunk-${chunks.id}
```

creates tasks:

```
task-chunk-1:
  command: "task-run --chunk=1 --totalChunks=2"
task-chunk-2:
  command: "task-run --chunk=2 --totalChunks=2"
```

# Overall processing model

The overall processing model for tasks is as follows:
 * Evaluate maps
 * Perform subsitutions
 * Evaluate use statements
 * Expand chunks
 * Perform subsitutions

At each point after maps are evaluated tasks must have a unique name.
