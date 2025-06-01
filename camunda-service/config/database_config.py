import os


class DatabaseConfig:
    DB_PATH = os.path.join(
        os.path.dirname(os.path.dirname(__file__)),
        "hoc_toc_data.db"
    )

    TIMEOUT = 30.0
    CHECK_SAME_THREAD = False
