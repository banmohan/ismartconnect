namespace ISmartConnect.Module.Models;

public class ResLoanPaymentInfo
{
    public decimal Interest { get; set; }
    public decimal Fine { get; set; }
    public decimal Penalty { get; set; }
    public decimal IntDiscount { get; set; }
    public decimal EmiDiscount { get; set; }
    public decimal PrincipalDueInstallment { get; set; }
    public decimal PaidPrincipal { get; set; }
    public decimal TotalPayable { get; set; }
}

public class ReqLoanPaymentDetail : ReqAccount
{
    public string? MobileNumber { get; set; }
}