use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// 目标地址
    #[arg(short, long)]
    pub(crate) url: String,

    /// 持续时间（秒）
    #[arg(short, long, default_value_t = 1)]
    pub(crate) duration_secs: u64,

    /// 并发数
    #[arg(short, long, default_value_t = 1)]
    pub(crate) concurrent_requests: i32,

    /// 超时时间（秒）
    #[arg(long, default_value_t = 0)]
    pub(crate) timeout: u64,

    /// 打印详情
    #[arg(short, long, default_value_t = false)]
    pub(crate) verbose: bool,

    /// 请求方法
    #[arg(short, long, default_value = "GET")]
    pub(crate) method: String,

    /// json
    #[arg(short, long, default_value = "")]
    pub(crate) json: String,

    /// form表单
    #[arg(short, long, default_value = "")]
    pub(crate) form: String,
}
