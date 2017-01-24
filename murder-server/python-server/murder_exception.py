class MurderError(Exception):
    def __init__(self, error):
        super(MurderError, self).__init__(
            "{}: {}".format(
                error.error,
                error.details
            )
        )
