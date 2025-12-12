namespace ISmartConnect.Module.Models;

public class ResFundTransfer
{
    public string? TranCode { get; set; }
    public string? TranId { get; set; }
}

public class ReqFundTransfer
{
    public string? SrcBranchId { get; set; }
    public string? SrcAccount { get; set; }
    public string? SrcAccountType { get; set; }
    public string? DestBranchId { get; set; }
    public string? DestAccount { get; set; }
    public string? DestAccountType { get; set; }
    public string? Description1 { get; set; }
    public string? Description2 { get; set; }
    public string? Description3 { get; set; }
    public string? AppTranCode { get; set; }
    public DateOnly TranDate { get; set; }
    public decimal Amount { get; set; }
}