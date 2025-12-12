using ISmartConnect.Module.Contracts;

namespace ISmartConnect;

public class MicroserviceMeta(IConfiguration configuration) : IMicroServiceMeta
{
    public string ApiKey => configuration.GetSection("MicroService:ApiKey").Value ??
                            throw new NotImplementedException("API Key is not configured.");

    public string AccountServiceUrl => configuration.GetSection("MicroService:Account").Value ??
                                       throw new NotImplementedException(
                                           "Account service URL is not configured.");

    public string DepositServiceUrl => configuration.GetSection("MicroService:Deposit")?.Value ??
                                       throw new NotImplementedException(
                                           "Deposit service URL is not configured.");

    public string LoanServiceUrl => configuration.GetSection("MicroService:Loan")?.Value ??
                                    throw new NotImplementedException("Loan service URL is not configured.");

    public string CacheServiceUrl => configuration.GetSection("MicroService:Cache")?.Value ??
                                     throw new NotImplementedException("Cache service URL is not configured.");
}