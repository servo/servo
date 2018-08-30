""" Taskcluster client exceptions """


class TaskclusterFailure(Exception):
    """ Base exception for all Taskcluster client errors"""
    pass


class TaskclusterRestFailure(TaskclusterFailure):
    """ Failures in the HTTP Rest API """
    def __init__(self, msg, superExc, status_code=500, body={}):
        TaskclusterFailure.__init__(self, msg)
        self.superExc = superExc
        self.status_code = status_code
        self.body = body


class TaskclusterConnectionError(TaskclusterFailure):
    """ Error connecting to resource """
    def __init__(self, msg, superExc):
        TaskclusterFailure.__init__(self, msg, superExc)
        self.superExc = superExc


class TaskclusterAuthFailure(TaskclusterFailure):
    """ Invalid Credentials """
    def __init__(self, msg, superExc=None, status_code=500, body={}):
        TaskclusterFailure.__init__(self, msg)
        self.superExc = superExc
        self.status_code = status_code
        self.body = body


class TaskclusterTopicExchangeFailure(TaskclusterFailure):
    """ Error while creating a Topic Exchange routing key """
    pass
