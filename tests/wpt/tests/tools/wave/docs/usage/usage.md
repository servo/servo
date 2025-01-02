# Usage Guide - [WAVE Test Runner](../README.md)

With WAVE Test Runner v1.0.0 all files and REST API endpoints are served under 
a configurable namespace, by default `/_wave/`, which will be used in this 
usage guide.

In this document the usage is explained using screenshots from the context of 
the WMAS project. However, practices can be applied to different contexts as well.

## Contents

1. [Creating test sessions](#1-creating-test-sessions)
   1. [The landing page](#11-the-landing-page)
   2. [Configuring a new session](#12-configuring-a-new-session)
   3. [Exclude tests](#13-exclude-tests)
      1. [Manually specify tests to exclude](#131-manually-specify-tests-to-exclude)
      2. [Use a session's malfunctioning list to add tests to exclude](#132-use-a-sessions-malfunctioning-list-to-add-tests-to-exclude)
      3. [Use a previous session's exclude list to add tests to exclude](#133-use-a-previous-sessions-exclude-list-to-add-tests-to-exclude)
2. [Resuming test sessions](#2-resuming-test-sessions)
   1. [Using the webinterface](#21-using-the-webinterface)
   2. [Using a URL](#22-using-a-url)
3. [Monitoring test sessions](#3-monitoring-test-sessions)
4. [Managing test sessions](#4-managing-test-sessions)

# 1. Creating test sessions

Test sessions hold information about one test run one a particular device, like the current status.
Each session is identified using a UUIDv1 token string to gather these information or perform actions on it.
Each new session is configured using several parameters before the run starts.

## 1.1 The landing page

Every new session is created from the landing page.
It is recommended to create a new session from the device that is tested, as the user agent is part of the displayed information, as well as the browser and version, which gets parsed from it.
However, this does not influence the execution of tests or the creation of test results.  
To create a new session, open the landing page on the URI path `/_wave/index.html`.

![landing_page]

The landing page is divided into two section, one to create a new session and one to resume a session.  
As soon as the landing is opened, a new test session is created.
Its token is displayed next to the QR-Code on the right, along with the expiration date.
As the session was created automatically, it gets removed automatically once it expires.
However, if you start the session, the expiration date gets removed and the sessions is available until you delete it.

## 1.2 Configuring a new session

To configure and start the session, either click on "Configure Session" or scan the QR-Code.
In most cases it is recommended to scan the QR-Code, as it does not require any interaction with the landing page on the DUT.

![configuration_page]

In the configuration screen you can set parameters for the new session and start it.  
At the top the session's token and expiration date is displayed. Next there is the "Labels" option, which allows adding any number of labels to the session, helping to better organize sessions and allowing to apply filters while searching.
Labels can be added and modified at any point in the future.  
Next there is the API selection, which allows defining the set of APIs to test in the new session. To exclude specific test or subdirectories of those selected APIs, there is the "Excluded Tests" option right below it. Here you can specify what tests to exclude in three distinct ways. (More details in [1.3 Exclude tests](#13-exclude-tests))

![configuration_page_bottom]

With the "Test Types" option you specify what types of test should be included into the session: in contrast to automatic tests, manual tests require user interaction to execute properly.
The "Reference Browsers" option lets you select browsers that are used to further filter the set of tests included in the session.
Only tests that have passed the reference test session in all selected browsers are included.
The reference browsers represent the status of implementation of all WAVE APIs in modern desktop browsers, at about the time the WAVE specification was published.  
To start the session press "Start Session", note that the landing page has to stay opened, as the test are going to be execute in the same window.

[To the top](#usage-guide---wave-test-runner)

## 1.3 Exclude tests

To have a fine control over what test cases are executed when configuring a session, it is possible to provide a list of test cases, that are omitted in the run.

### 1.3.1 Manually specify tests to exclude

To add tests to exclude by providing a plain text list, click on "Add Raw" in the "Excluded Tests" setting.
This opens a input field, where you can enter multiple full paths to test files or directories.

![Exclude List Add Raw][configuration_page_add_raw]

Each line will be interpreted as a path to exclude a single or a group of tests.
All tests that have a path starting with one of the provided, will be excluded in the session.
Lines starting with a # symbol will be ignored, in case you want to organize test paths in a text file using comments.
Click "Add" and you will see the paths listed in the table below.

### 1.3.2 Use a session's malfunctioning list to add tests to exclude

When flagging tests in a running session as malfunctioning, e.g. when crashing the device, it is possible to add these test to the exclude list of the new session.
To do this, click on "Add Malfunctioning" in the "Excluded Tests" section.

![Exclude List Add Malfunctioning][configuration_page_add_malfunctioning]

Enter the first eight characters or more into the text field labelled "Session Token" to import all tests from the session's malfunctioning list into the new session's exclude list.
Click "Add" to confirm.
The tests should now appear in the list below.

### 1.3.3 Use a previous session's exclude list to add tests to exclude

If you have already specified a suitable exclude list or want to expand an existing, you can apply the exclude list of a previous session.
Click on "Add Previous Excluded" in the "Excluded Tests" section to open the corresponding controls.

![Exclude List Add Previously Excluded][configuration_page_add_prev_excluded]

Enter the first eight characters or more into the text field labelled "Session Token" to import all tests from the previous session's exclude list into the new session's exclude list.
Click "Add" to confirm.
The tests should now appear in the list below.

[To the top](#usage-guide---wave-test-runner)

# 2. Resuming test sessions

Certain test cases may cause some devices to crash, which makes the test runner unable to automatically run the next test.
In this case, external interaction is necessary.
To alleviate the process of resuming the test session, the are two mechanisms integrated into the web interface that reduce interaction with the device to a minimum.
There is also a mechanism that can be useful if a test framework with access to the tested browser is utilized.

## 2.1 Using the webinterface

In any case, it is necessary to open the landing page on the device, in order to resume the session.

![Landing Page][landing_page]

On the landing page, in the section "Resume running session", you can see the token of the last session this device has run.
To resume this particular session, click on the "Resume" button next to it, or simply press enter or space.
If the presented token is not the one of the session you want to resume, you can change it from the configuration screen.
To get there, press the "Configure Session" button or scan the QR-Code.

![Configuration Page][configuration_page]

At the very bottom of the configuration page, there is a section called "Resume session", where you can see the token that was previously displayed on the landing page in a text box.
Here you can change the token of the session to resume, just enter the first eight characters or more of the token.
When you're done, press the "Resume" button.
Note that it is necessary to keep the landing page open in order to automatically run the next test, as it is loaded in the same window.

## 2.2 Using a URL

If you have access to the DUTs browser programmatically, you may want to resume a crashed test session automatically.
To load the next test of a specific session, simply open the following URL:

`/next.html?token=<session_token>`

For example:

`/_wave/next.html?token=24fcd360-ef4d-11e9-a95f-d6e1ad4c5fdb`

[To the top](#usage-guide---wave-test-runner)

# 3. Monitoring test sessions

While running test sessions, the results page for second screen devices provide a convenient summary of the sessions current state, as well as controls to manipulate the test execution.
Additionally, you can flag tests in case they interrupt the test execution by, e.g. crashing the test, to exclude them in future sessions and download test results and reports.

![results_page_top]

On the top right-hand side, there are controls to stop, pause or delete the session.
Stopping, as well as deleting the session is irreversible.
Below you find the session's details, including the token, user agent, test paths, excluded test paths, total test file count, status, the different test timeouts, the date and time the session has been started, the date and time the session has finished, the duration and labels.

![results_page_last_completed]

Right below, tests that have recently completed with result status TIMEOUT are listed to add them to the list of malfunctioning tests by clicking the button with the + symbol.
Now that test appears in the list of malfunctioning tests at the very bottom of the result page.
This list can be used to exclude tests when creating a new session. (more details in [1.3.2 Use a session's malfunctioning list to add tests to exclude](#132-use-a-sessions-malfunctioning-list-to-add-tests-to-exclude))

![results_page_api_results]

In the section "API Results" you can see the progress of each individual API selected for the session.
As each test file can contain multiple subtests, the count of passed, failed, timed out and not run tests does not correlate to the count of test files run, which indicates the overall progress.
Keep in mind that only test files that received a result will count as run, so even if all tests finished executing on the device, some may have failed to send the result, in which case the internal timeout has to run out to create it.

![results_page_api_results_export]

Once all test files of an API have received a result, it is possible to download the result data or view a report for that API, by clicking the corresponding button in the far right column of the table.

![results_page_bottom]

Below the table of API results, there are more options to download the results of the session.
The first option downloads the results the same way it is persisted on the serverside, along with some meta data.
This form is especially useful if you want to import the session details with the results into other instances of the WAVE Test Runner.  
Furthermore, there is the option to download the raw result in JSON format of all finished APIs.
This the same JSON you get by clicking on the "JSON" button in the API results column, but of all finished APIs in a ZIP file.
Lastly, you can download a static HTML page, similiar to the results view.  
Finally, at the bottom of the page you can find the list of malfunctioning tests that have been added from the list of last timed-out test files.
Remove tests by clicking their corresponding button with the trashcan icon.

[To the top](#usage-guide---wave-test-runner)

# 4. Managing test sessions

The overview page provides features that help to manage and organize multiple sessions. You can access it from the URL `/_wave/overview.html`.

![overview_page]

In the "Manage Sessions" section you can add more sessions to the list below by entering the first eight or more characters of the token.
Clicking on "Add Session" will add the session to the list if it was the only one that could be associated with the provided token.
If there are multiple sessions that match the provided input, none will be added.  
Additionally, you can compare multiple session, given that they are completed, used the same reference sessions and share tested APIs.
Simply select the desired session from the list below and click "Compare Selected".  
You can also import sessions in the "Import Sessions" section, however, this feature has to be enabled in the server configuration.  
Below the "Manage Sessions" section, there is the list of reference and recent sessions.

![overview_page_sessions]

In the sessions list, sessions are organized in three lists: Reference Browsers, which are test results everyone can see, containing the results of the reference browsers for the corresponding WAVE specification, recent sessions, which are sessions there have recently been viewed or executed on the device, and pinned sessions, which are sessions pinned by the user from the list of recent sessions.
Add label filters to show only matching sessions.

![overview_page_sessions_pinned_recent]

You can pin a session by clicking the button with the tag on a session in the recent sessions list and unpin them the same way from the pinned sessions list.
Click the trashcan icon to remove a session from its list, this will not delete the session results.
Sort the list of sessions by clicking on the column to filter them by.

![overview_page_sessions_filtered]

Add one or more tags to the filter to conveniently find the sessions you are looking for. Add labels to session when creating them or in their corresponding results page.

[To the top](#usage-guide---wave-test-runner)

[landing_page]: ../res/landing_page.jpg "Landing Page"
[configuration_page]: ../res/configuration_page_top.jpg "Configuration Page"
[configuration_page_bottom]: ../res/configuration_page_bottom.jpg "Configuration Page"
[configuration_page_add_raw]: ../res/configuration_page_exclude_add_raw.jpg "Exclude Tests - Add Raw"
[configuration_page_add_malfunctioning]: ../res/configuration_page_exclude_add_malfunctioning.jpg "Exclude Tests - Add Malfunctioning"
[configuration_page_add_prev_excluded]: ../res/configuration_page_exclude_add_prev_excluded.jpg "Exclude Tests - Add Previously Excluded"
[results_page_top]: ../res/results_page_top.jpg "Results Page"
[results_page_last_completed]: ../res/results_page_last_timed_out.jpg "Results Page"
[results_page_api_results]: ../res/results_page_api_results.jpg "Results Page"
[results_page_api_results_export]: ../res/results_page_api_results_export.jpg "Results Page"
[results_page_bottom]: ../res/results_page_bottom.jpg "Results Page"
[overview_page]: ../res/overview_page_top.jpg "Overview Page"
[overview_page_sessions]: ../res/overview_page_sessions.jpg "Overview Page Sessions"
[overview_page_sessions_pinned_recent]: ../res/overview_page_sessions_pinned_recent.jpg "Overview Page Sessions"
[overview_page_sessions_filtered]: ../res/overview_page_sessions_filtered.jpg "Overview Page Filter"
