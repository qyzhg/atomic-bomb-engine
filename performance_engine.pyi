def run(
        url: str,
                method: str,
                test_duration_secs: int,
                concurrent_requests: int,
                timeout_secs: int,
                verbose: bool = False,
                json_str: str | None = None,
                form_data_str: str | None = None,
                headers: list[str] | None = None,
                cookie: str | None = None) -> dict:
        """
        同步启动压测引擎
        :param url: 压测地址
        :param method: 请求方式
        :param test_duration_secs: 持续时间
        :param concurrent_requests: 并发量
        :param timeout_secs: 接口超时时间
        :param verbose: 开启详情日志
        :param json_str: 使用json请求发送请求,使用json字符串,不要使用字典类型
        :param form_data_str: 使用form方式发送请求
        :param headers: 添加请求头
        :param cookie: 添加cookie
        :return:
        """

def run_async(
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
        异步启动压测引擎
        :param url: 压测地址
        :param method: 请求方式
        :param test_duration_secs: 持续时间
        :param concurrent_requests: 并发量
        :param timeout_secs: 接口超时时间
        :param verbose: 开启详情日志
        :param json_str: 使用json请求发送请求,使用json字符串,不要使用字典类型
        :param form_data_str: 使用form方式发送请求
        :param headers: 添加请求头
        :param cookie: 添加cookie
        :return:
        """

class StatusListenIter: iter
