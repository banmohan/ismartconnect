namespace ISmartConnect.Module.Models;

public class ResBalance
{
    public decimal LedgerBalance { get; set; }
    public decimal AvailableBalance { get; set; }
}

public record PremiumCbsBalResModel
{
    public long Id { get; set; }
    public string AccountNumber { get; set; }
    public decimal Balance { get; set; }
    public decimal Dp { get; set; }
}

