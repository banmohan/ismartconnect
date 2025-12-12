namespace ISmartConnect.Module.Models;

public class ResMiniStatement
{
    public decimal AvailableBalance { get; set; }
    public IEnumerable<StatementItem>? Data { get; set; }
}
public class StatementItem
{
    public long TranId { get; set; }
    public DateTime TranDate { get; set; }
    public string? TranType { get; set; } // Dr | Cr
    public decimal? Amount { get; set; }
    public string? Remarks { get; set; }
}

public record PremiumCbsStmtItemModel
{
    public long TranId { get; set; }
    public DateTime ValueDate { get; set; }
    public decimal Debit { get; set; }
    public decimal Credit { get; set; }
    public string StatementReference { get; set; }
}