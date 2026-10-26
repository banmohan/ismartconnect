use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};

/// Swagger UI only attaches an `Authorization` header from a security scheme.
/// A header parameter of the same name is shown and then dropped.
struct ClientKeySecurity;

impl Modify for ClientKeySecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "client_api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "Authorization",
                "Raw client API key. Paste the key only, with no Bearer prefix.",
            ))),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&ClientKeySecurity),
    security(("client_api_key" = [])),
    info(
        title = "MobileApi",
        version = "1.0.0",
        description = "Mobile banking middleware in front of PremiumCBS. Click Authorize and paste the client API key. Swagger sends it as the Authorization header. Also send the matching client code in the tenant header. Success responses are camelCase and include isoResponseCode 00. CBS failures return HTTP 500 with message, errorCode 500, and an isoResponseCode."
    ),
    paths(
        crate::routes::echo::echo,
        crate::routes::deposit::balance,
        crate::routes::deposit::mini_statement,
        crate::routes::deposit::full_statement,
        crate::routes::deposit::fund_transfer,
        crate::routes::deposit::reversal,
        crate::routes::deposit::acc_validation,
        crate::routes::deposit::all_account,
        crate::routes::loan::loan_detail,
        crate::routes::loan::loan_statement,
        crate::routes::loan::loan_schedule
    ),
    tags(
        (name = "echo", description = "Connectivity"),
        (name = "deposit", description = "Deposit accounts and transfers"),
        (name = "loan", description = "Loan accounts")
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::ApiDoc;
    use utoipa::OpenApi;

    #[test]
    fn authorization_is_sent_as_an_api_key_header() {
        let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let scheme = &doc["components"]["securitySchemes"]["client_api_key"];
        assert_eq!(scheme["type"], "apiKey");
        assert_eq!(scheme["in"], "header");
        assert_eq!(scheme["name"], "Authorization");
        assert_eq!(doc["security"][0]["client_api_key"], serde_json::json!([]));
    }
}
