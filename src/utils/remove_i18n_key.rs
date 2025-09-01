use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use serde_json::{json, Value};
use std::error::Error;

pub struct RemoveI18nKeyConfig {
    pub module_id: u32,
    pub route_id: u32,
    pub operate: String,
}

pub struct RemoveI18nKeyResult {
    pub ids: Vec<u32>,
    pub unknowns: Vec<String>,
}

pub async fn remove_i18n_keys(
    config: RemoveI18nKeyConfig,
    keys: Vec<String>,
) -> Result<RemoveI18nKeyResult, Box<dyn Error>> {
    let client = Client::new();
    let mut ids = Vec::new();
    let mut unknowns = Vec::new();

    // 构建请求头
    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert("accept-language", HeaderValue::from_static("zh,zh-CN;q=0.9,ja;q=0.8"));
    headers.insert("cache-control", HeaderValue::from_static("no-cache"));
    headers.insert("content-type", HeaderValue::from_static("application/json;charset=UTF-8"));
    headers.insert("pragma", HeaderValue::from_static("no-cache"));
    headers.insert("priority", HeaderValue::from_static("u=1, i"));
    headers.insert("sec-ch-ua", HeaderValue::from_static("\"Not)A;Brand\";v=\"8\", \"Chromium\";v=\"138\", \"Google Chrome\";v=\"138\""));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"macOS\""));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));

    // 查询每个 key 对应的 id
    for key in &keys {
        let query_body = json!({
            "pageSize": 10,
            "pageIndex": 1,
            "routeId": config.route_id.to_string(),
            "search": key,
            "searchMode": -1,
            "moduleId": config.module_id,
            "mtEngine": "ai-mt",
            "target": [{
                "moduleId": config.module_id,
                "routeId": config.route_id
            }],
            "operate": config.operate
        });

        match query_translation_id(&client, &headers, &query_body).await {
            Ok(id) => ids.push(id),
            Err(_) => {
                unknowns.push(key.clone());
                eprintln!("Failed to find id for key: {}", key);
            }
        }
    }

    println!("未找到id的标识：{:?}", unknowns);

    // 如果有找到的 ids，执行删除操作
    if !ids.is_empty() {
        let delete_body = json!({
            "routeId": config.route_id.to_string(),
            "id": ids,
            "identifier": format!("INTL_LANGPKG_{}", config.route_id),
            "moduleId": config.module_id,
            "target": [{
                "moduleId": config.module_id,
                "routeId": config.route_id
            }],
            "operate": config.operate
        });

        if let Err(e) = delete_translations(&client, &headers, &delete_body).await {
            eprintln!("Failed to delete translations: {}", e);
        }
    }

    Ok(RemoveI18nKeyResult { ids, unknowns })
}

async fn query_translation_id(
    client: &Client,
    headers: &HeaderMap,
    body: &Value,
) -> Result<u32, Box<dyn Error>> {
    let response = client
        .post("https://lingo.woa.com/polaris/api/langpkg/queryTranslationList")
        .headers(headers.clone())
        .header("referer", "https://lingo.woa.com/")
        .json(body)
        .send()
        .await?;

    let json_response: Value = response.json().await?;
    
    let id = json_response
        .get("data")
        .and_then(|data| data.get("translations"))
        .and_then(|translations| translations.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("id"))
        .and_then(|id| id.as_u64())
        .ok_or("Failed to extract id from response")?;

    Ok(id as u32)
}

async fn delete_translations(
    client: &Client,
    headers: &HeaderMap,
    body: &Value,
) -> Result<(), Box<dyn Error>> {
    let response = client
        .post("https://lingo.woa.com/polaris/api/langpkg/deleteTranslation")
        .headers(headers.clone())
        .header("referer", "https://lingo.woa.com/")
        .json(body)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Successfully deleted translations");
    } else {
        eprintln!("Failed to delete translations: {}", response.status());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_remove_i18n_keys() {
        let config = RemoveI18nKeyConfig {
            module_id: 2954,
            route_id: 3204,
            operate: "parkeryu".to_string(),
        };

        let keys = vec![
            // 在这里添加测试用的 key
        ];

        match remove_i18n_keys(config, keys).await {
            Ok(result) => {
                println!("Found IDs: {:?}", result.ids);
                println!("Unknown keys: {:?}", result.unknowns);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}