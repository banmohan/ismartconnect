use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::serde_util::{dec, dec_loose, dec_opt, flex_datetime};

fn iso_ok() -> String {
    "00".to_string()
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EchoResponse {
    pub result: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReqAccount {
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub account_number: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CbsBalance {
    #[serde(default, with = "dec_loose")]
    pub available_balance: Decimal,
    #[serde(default, with = "dec_loose")]
    pub ledger_balance: Decimal,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub available_balance: Decimal,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub ledger_balance: Decimal,
    pub iso_response_code: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatementItem {
    #[serde(default)]
    pub tran_id: i64,
    #[serde(deserialize_with = "flex_datetime")]
    pub tran_date: String,
    #[serde(default)]
    pub tran_type: Option<String>,
    #[serde(default, with = "dec_opt")]
    #[schema(value_type = Option<f64>)]
    pub amount: Option<Decimal>,
    #[serde(default)]
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CbsMiniStatement {
    #[serde(default, with = "dec_loose")]
    pub available_balance: Decimal,
    #[serde(default)]
    pub data: Vec<StatementItem>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MiniStatementResponse {
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub available_balance: Decimal,
    pub statement_list: Vec<StatementItem>,
    pub iso_response_code: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReqFullStatement {
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub account_number: Option<String>,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatementListResponse {
    pub statement_list: Vec<StatementItem>,
    pub iso_response_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoanStatementListResponse {
    pub statement_list: Vec<LoanStatementItem>,
    pub iso_response_code: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReqFundTransfer {
    #[serde(default)]
    pub src_branch_id: Option<String>,
    #[serde(default)]
    pub src_account: Option<String>,
    #[serde(default)]
    pub src_account_type: Option<String>,
    #[serde(default)]
    pub dest_branch_id: Option<String>,
    #[serde(default)]
    pub dest_account: Option<String>,
    #[serde(default)]
    pub dest_account_type: Option<String>,
    #[serde(default)]
    pub description1: Option<String>,
    #[serde(default)]
    pub description2: Option<String>,
    #[serde(default)]
    pub description3: Option<String>,
    #[serde(default)]
    pub app_tran_code: Option<String>,
    pub tran_date: NaiveDate,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub amount: Decimal,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FundTransferBody {
    #[serde(default)]
    pub tran_code: Option<String>,
    #[serde(default)]
    pub tran_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FundTransferResponse {
    pub tran_code: Option<String>,
    pub iso_response_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CbsAccountValidation {
    #[serde(default, with = "dec_loose")]
    pub ledger_balance: Decimal,
    #[serde(default, with = "dec_loose")]
    pub available_balance: Decimal,
    #[serde(default)]
    pub account_type: Option<String>,
    #[serde(default, with = "dec_loose")]
    pub accrued_interest: Decimal,
    #[serde(default)]
    pub acc_status: bool,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub customer_code: Option<String>,
    #[serde(default)]
    pub customer_name: Option<String>,
    #[serde(default)]
    pub date_of_birth: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub id_issue_date: Option<String>,
    #[serde(default)]
    pub id_issue_place: Option<String>,
    #[serde(default)]
    pub id_number: Option<String>,
    #[serde(default, with = "dec_loose")]
    pub interest_rate: Decimal,
    #[serde(default)]
    pub id_type: Option<String>,
    #[serde(default, with = "dec_loose")]
    pub min_balance: Decimal,
    #[serde(default)]
    pub mobile_number: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub fathers_name: Option<String>,
    #[serde(default)]
    pub mothers_name: Option<String>,
    #[serde(default)]
    pub grand_fathers_name: Option<String>,
    #[serde(default)]
    pub spouse_name: Option<String>,
    #[serde(default)]
    pub occupation: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountValidationResponse {
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub ledger_balance: Decimal,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub available_balance: Decimal,
    pub account_type: Option<String>,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub accrued_interest: Decimal,
    pub acc_status: bool,
    pub address: Option<String>,
    pub branch_id: Option<String>,
    pub customer_code: Option<String>,
    pub customer_name: Option<String>,
    pub date_of_birth: Option<String>,
    pub gender: Option<String>,
    pub id_issue_date: Option<String>,
    pub id_issue_place: Option<String>,
    pub id_number: Option<String>,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub interest_rate: Decimal,
    pub id_type: Option<String>,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub min_balance: Decimal,
    pub mobile_number: Option<String>,
    pub email: Option<String>,
    pub fathers_name: Option<String>,
    pub mothers_name: Option<String>,
    pub grand_fathers_name: Option<String>,
    pub spouse_name: Option<String>,
    pub occupation: Option<String>,
    pub iso_response_code: String,
}

impl AccountValidationResponse {
    pub fn from_cbs(src: CbsAccountValidation) -> Self {
        Self {
            ledger_balance: src.ledger_balance,
            available_balance: src.available_balance,
            account_type: src.account_type,
            accrued_interest: src.accrued_interest,
            acc_status: src.acc_status,
            address: src.address,
            branch_id: src.branch_id,
            customer_code: src.customer_code,
            customer_name: src.customer_name,
            date_of_birth: src.date_of_birth,
            gender: src.gender,
            id_issue_date: src.id_issue_date,
            id_issue_place: src.id_issue_place,
            id_number: src.id_number,
            interest_rate: src.interest_rate,
            id_type: src.id_type,
            min_balance: src.min_balance,
            mobile_number: src.mobile_number,
            email: src.email,
            fathers_name: src.fathers_name,
            mothers_name: src.mothers_name,
            grand_fathers_name: src.grand_fathers_name,
            spouse_name: src.spouse_name,
            occupation: src.occupation,
            iso_response_code: iso_ok(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepositAcc {
    #[serde(default)]
    pub member_code: String,
    #[serde(default)]
    pub account_number: String,
    #[serde(default)]
    pub branch_id: String,
    #[serde(default)]
    pub open_date: String,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub account_type: String,
    #[serde(default)]
    pub inst_freq: String,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub inst_amount: Decimal,
    #[serde(default)]
    pub maturity_date: String,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub int_rate: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub accrued_interest: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub balance: Decimal,
    #[serde(default, with = "dec_opt")]
    #[schema(value_type = Option<f64>)]
    pub min_balance: Option<Decimal>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoanAcc {
    #[serde(default)]
    pub member_code: String,
    #[serde(default)]
    pub branch_id: String,
    #[serde(default)]
    pub account_number: String,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub disbursed_amt: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub balance: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub due_interest: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub due_installment: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub due_fine: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub interest_rate: Decimal,
    #[serde(default)]
    pub issued_on: String,
    #[serde(default)]
    pub matures_on: String,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub account_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShareAcc {
    #[serde(default)]
    pub member_code: String,
    #[serde(default)]
    pub account_number: String,
    #[serde(default)]
    pub open_date: String,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub balance: Decimal,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub account_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CbsAllAccount {
    #[serde(default)]
    pub deposit: Vec<DepositAcc>,
    #[serde(default)]
    pub loan: Vec<LoanAcc>,
    #[serde(default)]
    pub share: Vec<ShareAcc>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AllAccountResponse {
    pub deposit: Vec<DepositAcc>,
    pub loan: Vec<LoanAcc>,
    pub share: Vec<ShareAcc>,
    pub iso_response_code: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReqLoanDetail {
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub account_number: Option<String>,
    #[serde(default)]
    pub customer_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CbsLoanDetails {
    #[serde(default)]
    pub member_code: String,
    #[serde(default)]
    pub branch_id: String,
    #[serde(default)]
    pub account_number: String,
    #[serde(default, with = "dec_loose")]
    pub disbursed_amt: Decimal,
    #[serde(default, with = "dec_loose")]
    pub balance: Decimal,
    #[serde(default, with = "dec_loose")]
    pub due_interest: Decimal,
    #[serde(default, with = "dec_loose")]
    pub due_installment: Decimal,
    #[serde(default, with = "dec_loose")]
    pub due_fine: Decimal,
    #[serde(default, with = "dec_loose")]
    pub interest_rate: Decimal,
    #[serde(default)]
    pub issued_on: String,
    #[serde(default)]
    pub matures_on: String,
    #[serde(default)]
    pub product: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoanDetailResponse {
    pub branch_id: String,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub due_interest: Decimal,
    pub issued_on: String,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub due_installment: Decimal,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub due_fine: Decimal,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub balance: Decimal,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub disbursed_amt: Decimal,
    pub account_number: String,
    #[serde(with = "dec")]
    #[schema(value_type = f64)]
    pub interest_rate: Decimal,
    pub matures_on: String,
    pub member_code: String,
    pub product: String,
    pub iso_response_code: String,
}

impl LoanDetailResponse {
    pub fn from_cbs(src: CbsLoanDetails) -> Self {
        Self {
            branch_id: src.branch_id,
            due_interest: src.due_interest,
            issued_on: src.issued_on,
            due_installment: src.due_installment,
            due_fine: src.due_fine,
            balance: src.balance,
            disbursed_amt: src.disbursed_amt,
            account_number: src.account_number,
            interest_rate: src.interest_rate,
            matures_on: src.matures_on,
            member_code: src.member_code,
            product: src.product,
            iso_response_code: iso_ok(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReqLoanStatement {
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub account_number: Option<String>,
    #[serde(default)]
    pub customer_code: Option<String>,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoanStatementItem {
    pub tran_date: NaiveDate,
    pub interest_date: NaiveDate,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub issue_amount: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub payment: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub principal: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub interest: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub fine: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub discount: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub balance: Decimal,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoanScheduleItem {
    #[serde(default)]
    pub schedule_no: i32,
    #[serde(deserialize_with = "flex_datetime")]
    pub schedule_date: String,
    #[serde(default)]
    pub schedule_date_np: Option<String>,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub amount: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub principal: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub interest: Decimal,
    #[serde(default, with = "dec_loose")]
    #[schema(value_type = f64)]
    pub principal_balance: Decimal,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoanScheduleResponse {
    pub schedules: Vec<LoanScheduleItem>,
    pub iso_response_code: String,
}
