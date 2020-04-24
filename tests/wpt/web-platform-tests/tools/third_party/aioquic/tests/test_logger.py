from unittest import TestCase

from aioquic.quic.logger import QuicLogger


class QuicLoggerTest(TestCase):
    def test_empty(self):
        logger = QuicLogger()
        self.assertEqual(logger.to_dict(), {"qlog_version": "draft-01", "traces": []})

    def test_empty_trace(self):
        logger = QuicLogger()
        trace = logger.start_trace(is_client=True, odcid=bytes(8))
        logger.end_trace(trace)
        self.assertEqual(
            logger.to_dict(),
            {
                "qlog_version": "draft-01",
                "traces": [
                    {
                        "common_fields": {
                            "ODCID": "0000000000000000",
                            "reference_time": "0",
                        },
                        "configuration": {"time_units": "us"},
                        "event_fields": [
                            "relative_time",
                            "category",
                            "event_type",
                            "data",
                        ],
                        "events": [],
                        "vantage_point": {"name": "aioquic", "type": "client"},
                    }
                ],
            },
        )
