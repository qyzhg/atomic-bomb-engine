def run(
        url: str,
                method: str,
                test_duration_secs: int,
                concurrent_requests: int,
                timeout_secs: int,
                verbose: bool = False,
                json_str: str | None = None,
                form_data_str: str | None = None,
                headers: str | None = None,
                cookie: str | None = None) -> dict:
        """
            压测引擎
        """