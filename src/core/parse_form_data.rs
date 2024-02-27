use std::collections::HashMap;

// 将字符串解析成form数据
pub(crate) fn parse_form_data(form_data_str: &str) -> HashMap<String, String> {
    form_data_str.split('&')
        .filter_map(|part| {
            let mut parts = part.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                _ => None,
            }
        })
        .collect()
}