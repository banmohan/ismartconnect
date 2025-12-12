using ISmartConnect.Module.Intercom;

namespace ISmartConnect.Module.Contracts;

public interface IMicroServiceMeta
{
    string ApiKey { get; }
    string AccountServiceUrl { get; }
    string DepositServiceUrl { get; }
    string LoanServiceUrl { get; }
    string CacheServiceUrl { get; }

    string GetUrl(MicroService moduleName) =>
        moduleName switch
        {
            MicroService.Account => AccountServiceUrl,
            MicroService.Deposit => DepositServiceUrl,
            MicroService.Loan => LoanServiceUrl,
            MicroService.Cache => CacheServiceUrl,
            _ => "http://localhost"
        };
}