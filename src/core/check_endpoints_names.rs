use std::collections::HashSet;
use anyhow::{anyhow};
use crate::models::api_endpoint::ApiEndpoint;

pub(crate) fn check_endpoints_names(endpoints: Vec<ApiEndpoint>) -> anyhow::Result<()> {
    let mut names_set = HashSet::new();
    for endpoint in endpoints {
        if endpoint.name.clone().is_empty(){
            return Err(anyhow!("api名称不能为空"));
        }
        if !names_set.insert(endpoint.name.clone()) {
            return Err(anyhow!("重复的name: {}", endpoint.name));
        }
    }
    Ok(())
}