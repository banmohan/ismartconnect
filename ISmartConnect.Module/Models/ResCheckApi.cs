namespace ISmartConnect.Module.Models;

public class ResCheckApi
{
    public int ResponseCode { get; set; }
    public string Status { get; set; } = string.Empty;
    public string Message { get; set; } = string.Empty;
    public string? VoucherNo { get; set; }
    public string? VoucherDate { get; set; }
    public string? DebitAccount { get; set; }
    public string? CreditAccount { get; set; }
    public decimal Amount { get; set; }
    public string TransactionId { get; set; } = string.Empty;
}
