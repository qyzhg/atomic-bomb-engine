use clap_derive::Parser;
use clap::{ArgAction};
use serde_json::Value;

/// 轻量级高性能压测引擎
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// 目标地址
    #[arg(short, long, required = true)]
    pub(crate) url: String,

    /// 请求方法
    #[arg(short, long, default_value = "GET")]
    pub(crate) method: String,

    /// 持续时间（秒）
    #[arg(short, long, default_value_t = 1)]
    pub(crate) duration_secs: u64,

    /// 并发数
    #[arg(short, long, default_value_t = 1)]
    pub(crate) concurrent_requests: i32,

    /// 超时时间（秒）
    #[arg(long, default_value_t = 0)]
    pub(crate) timeout: u64,

    /// 打印请求详情,关闭进度条
    #[arg(short, long, default_value_t = false)]
    pub(crate) verbose: bool,

    /// json
    #[arg(short, long)]
    pub(crate) json: Option<String>,

    /// form表单(key1=val1&key2=val2...)
    #[arg(short, long)]
    pub(crate) form: Option<String>,

    /// 设置HTTP头部，使用英文冒号分割(key:val)多个请求头可以使用多个-H或者--header参数
    #[clap(short = 'H', long = "header", action = ArgAction::Append)]
    pub headers: Option<Vec<String>>,

    /// 设置cookie
    #[clap(short = 'C', long = "cookie")]
    pub cookie: Option<String>,
}
