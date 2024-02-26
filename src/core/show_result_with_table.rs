use prettytable::{Cell, format, row, Row, Table};
use crate::models::result::TestResult;

pub(crate) fn show_result_with_table(result: TestResult){
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    table.add_row(row!["指标", "值"]);
    table.add_row(row!["RPS", format!("{:.3}", result.rps)]);
    table.add_row(row!["总请求数", format!("{:?}", result.total_requests)]);
    table.add_row(row!["错误数量", format!("{:?}", result.err_count)]);
    table.add_row(row!["成功率", format!("{:.2}%", result.success_rate)]);
    table.add_row(row!["最大响应时间", format!("{:.2}ms", result.max_response_time)]);
    table.add_row(row!["最小响应时间", format!("{:.2}ms", result.min_response_time)]);
    table.add_row(row!["中位响应时间", format!("{} ms", result.median_response_time)]);
    table.add_row(row!["95%响应时间", format!("{} ms", result.response_time_95)]);
    table.add_row(row!["99%响应时间", format!("{} ms", result.response_time_99)]);
    table.add_row(row!["总吞吐量", format!("{:.2}kb", result.total_data_kb)]);
    println!("压测结果:");
    table.printstd();

    if !result.http_errors.is_empty() {
        let mut errors_table = Table::new();
        errors_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        errors_table.add_row(row!["错误代码", "错误信息", "次数"]);
        for e in result.http_errors {
            errors_table.add_row(Row::new(vec![
                Cell::new(format!("{:03}", e.0.0).as_str()),
                Cell::new(&format!("{:?}", e.0.1)).style_spec("R"),
                Cell::new(format!("{}", e.1).as_str()),
            ]));
        }
        println!("HTTP 错误:");
        errors_table.printstd();
    }

}