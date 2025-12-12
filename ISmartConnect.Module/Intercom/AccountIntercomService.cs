using ISmartConnect.Module.Contracts;
using ISmartConnect.Module.Models;

namespace ISmartConnect.Module.Intercom;

public class AccountIntercomService(IMicroServiceMeta microServiceMeta, IUserMeta userMeta)
    : BaseIntercomService(MicroService.Account, microServiceMeta, userMeta)
{
    public async Task<bool> EchoAsync()
    {
        const string url = "api/mbank/echo";
        return await Get<bool>(url);
    }

    public async Task<ResBalance> BalanceEnquiryAsync(ReqAccount model)
    {
        const string url = "api/mbank/balance";
        return await Post<ResBalance, ReqAccount>(url, model);
    }


    public async Task<ResMiniStatement> MiniStatementAsync(ReqAccount model)
    {
        const string url = "api/mbank/mini-statement";
        return await Post<ResMiniStatement, ReqAccount>(url, model);
    }

    public async Task<IEnumerable<StatementItem>> FullStatementAsync(ReqFullStatement model)
    {
        const string url = "api/mbank/full-statement";
        return await Post<List<StatementItem>, ReqFullStatement>(url, model);
    }

    public async Task<ResFundTransfer> FundTransferAsync(ReqFundTransfer model)
    {
        const string url = "api/mbank/fund-transfer";
        return await Post<ResFundTransfer, ReqFundTransfer>(url, model);
    }

    public async Task<bool> ReversalAsync(ResFundTransfer model)
    {
        const string url = "api/mbank/reversal";
        return await Post<bool, ResFundTransfer>(url, model);
    }

    public async Task<ResAccountValidation> AccValidateAsync(ReqAccount model)
    {
        const string url = "api/mbank/acc-validation";
        return await Post<ResAccountValidation, ReqAccount>(url, model);
    }

    public async Task<ResAllAccount> AllAccountAsync(string accountNumber)
    {
        var url = $"api/mbank/all-account/{accountNumber}";
        return await Get<ResAllAccount>(url);
    }

    public async Task<IEnumerable<ResLoanStatement>> LoanStatementAsync(ReqLoanStatement model)
    {
        const string url = "api/mbank/loan-statement";
        return await Post<List<ResLoanStatement>, ReqLoanStatement>(url, model);
    }

    public async Task<ResLoanDetails> LoanDetailAsync(ReqLoanDetail model)
    {
        const string url = "api/mbank/loan-detail";
        return await Post<ResLoanDetails, ReqLoanDetail>(url, model);
    }

    public async Task<IEnumerable<ResLoanSchedule>> LoanScheduleAsync(ReqAccount model)
    {
        const string url = "api/mbank/loan-schedule";
        return await Post<List<ResLoanSchedule>, ReqAccount>(url, model);
    }
}