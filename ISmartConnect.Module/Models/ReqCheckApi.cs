namespace ISmartConnect.Module.Models;

public class ReqCheckApi
{
    public string TransactionId { get; set; } = string.Empty;
    public string DebitAccountNo { get; set; } = string.Empty;
    public string CreditAccountNo { get; set; } = string.Empty;
    public decimal TransactionAmount { get; set; }
}
