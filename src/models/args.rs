use clap_derive::Parser;
use clap::{ArgAction};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// 目标地址
    #[arg(short, long, required = true)]
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
    pub(crate) json: Option<String>,

    /// form表单
    #[arg(short, long, default_value = "")]
    pub(crate) form: String,

    /// 设置HTTP头部
    #[clap(short = 'H', long = "header", action = ArgAction::Append)]
    pub headers: Vec<String>,

    /// 设置cookie
    #[clap(short = 'C', long = "cookie")]
    pub cookie: Option<String>,
}
