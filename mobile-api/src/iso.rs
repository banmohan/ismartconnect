/// Maps a PremiumCBS error message onto an ISO 8583 response code.
/// Exact messages match the middleware this service replaces. The EOD phrase is a
/// case-insensitive substring, because CBS appends detail to that sentence.
pub fn iso_from_message(message: &str) -> &'static str {
    match message {
        "InvalidAccount" => "76",
        "Branch is closed for EOD processing, Contact admin" => "39",
        "Day close process running" => "39",
        "InvalidTransaction" => "30",
        "TranNotAllowed" => "39",
        "InsufficientFund" => "51",
        "UnableToProcess" => "05",
        "AccountDormant" => "52",
        "AccountClosed" => "54",
        "AccountRestricted" => "62",
        "DuplicateReversal" => "98",
        _ if message.to_ascii_lowercase().contains("eod started") => "39",
        _ => "96",
    }
}

#[cfg(test)]
mod tests {
    use super::iso_from_message;

    #[test]
    fn maps_known_cbs_messages() {
        let cases = [
            ("InsufficientFund", "51"),
            ("Branch is closed for EOD processing, Contact admin", "39"),
            ("Day close process running", "39"),
            ("EOD started for branch 01", "39"),
            ("eod started", "39"),
            ("InvalidAccount", "76"),
            ("InvalidTransaction", "30"),
            ("TranNotAllowed", "39"),
            ("UnableToProcess", "05"),
            ("AccountDormant", "52"),
            ("AccountClosed", "54"),
            ("AccountRestricted", "62"),
            ("DuplicateReversal", "98"),
            ("something else", "96"),
            ("InsufficientFund extra", "96"),
        ];
        for (message, iso) in cases {
            assert_eq!(iso_from_message(message), iso, "{message}");
        }
    }
}
