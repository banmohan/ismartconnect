use rust_decimal::Decimal;
use serde_json::Value;

use crate::text::truncate;

#[derive(Debug, Default, PartialEq)]
pub struct Extracted {
    pub account_number: Option<String>,
    pub src_account: Option<String>,
    pub dest_account: Option<String>,
    pub member_id: Option<String>,
    pub tran_code: Option<String>,
    pub amount: Option<Decimal>,
    pub iso_response_code: Option<String>,
}

pub fn extract_fields(path: &str, request_body: &str, response_body: &str) -> Extracted {
    let request = serde_json::from_str::<Value>(request_body).unwrap_or(Value::Null);
    let response = serde_json::from_str::<Value>(response_body).unwrap_or(Value::Null);
    Extracted {
        account_number: text_at(&request, "accountNumber").or_else(|| text_at(&response, "accountNumber")),
        src_account: text_at(&request, "srcAccount"),
        dest_account: text_at(&request, "destAccount"),
        member_id: member_from_path(path).or_else(|| text_at(&request, "memberId")),
        tran_code: text_at(&response, "tranCode")
            .or_else(|| text_at(&request, "tranCode"))
            .or_else(|| text_at(&request, "appTranCode")),
        amount: decimal_at(&request, "amount"),
        iso_response_code: text_at(&response, "isoResponseCode"),
    }
}

pub fn build_search_text(path: &str, error: Option<&str>, client: Option<&str>, extracted: &Extracted) -> String {
    let parts = [
        Some(path),
        error,
        client,
        extracted.account_number.as_deref(),
        extracted.src_account.as_deref(),
        extracted.dest_account.as_deref(),
        extracted.member_id.as_deref(),
        extracted.tran_code.as_deref(),
        extracted.iso_response_code.as_deref(),
    ];
    let text = parts
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    truncate(&text, 4000)
}

fn member_from_path(path: &str) -> Option<String> {
    let marker = "/all-account/";
    let rest = path.split(marker).nth(1)?;
    let id = rest.split(['/', '?']).next().unwrap_or("");
    if id.is_empty() { None } else { Some(id.to_string()) }
}

fn text_at(value: &Value, key: &str) -> Option<String> {
    match value.get(key)? {
        Value::String(text) if !text.is_empty() => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn decimal_at(value: &Value, key: &str) -> Option<Decimal> {
    match value.get(key)? {
        Value::Number(number) => number.to_string().parse().ok(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_search_text, extract_fields};

    #[test]
    fn extracts_fund_transfer_fields() {
        let extracted = extract_fields(
            "/api/deposit/fund-transfer",
            r#"{"srcAccount":"A","destAccount":"B","amount":150.25,"appTranCode":"APP1"}"#,
            r#"{"tranCode":"T9","isoResponseCode":"00"}"#,
        );
        assert_eq!(extracted.src_account.as_deref(), Some("A"));
        assert_eq!(extracted.dest_account.as_deref(), Some("B"));
        assert_eq!(extracted.tran_code.as_deref(), Some("T9"));
        assert_eq!(extracted.iso_response_code.as_deref(), Some("00"));
        assert_eq!(extracted.amount.unwrap().to_string(), "150.25");
        let search = build_search_text("/api/deposit/fund-transfer", None, Some("dev"), &extracted);
        assert!(search.contains("A"));
        assert!(search.contains("T9"));
        assert!(search.contains("dev"));
    }

    #[test]
    fn extracts_member_from_all_account_path() {
        let extracted = extract_fields("/api/deposit/all-account/4451", "", r#"{"isoResponseCode":"00"}"#);
        assert_eq!(extracted.member_id.as_deref(), Some("4451"));
    }
}
