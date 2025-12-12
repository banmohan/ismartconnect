namespace ISmartConnect.Module.Models;

public class ResLoanDetails
{
    // public string? ProductName { get; set; }
    // public string? AccountNumber { get; set; }
    // public decimal InterestRate { get; set; }
    // public DateTime IssuedOn { get; set; }
    // public DateTime MaturesOn { get; set; }
    // public int Duration { get; set; }
    // public string? InterestType { get; set; }
    // public decimal DisburseAmount { get; set; }
    // public decimal Balance { get; set; }
    // public int IntInstallments { get; set; }
    // public int PrincipalInstallments { get; set; }
    
    public string MemberCode { get; set; }
    public string BranchId { get; set; }
    public string AccountNumber { get; set; }
    public decimal DisbursedAmt { get; set; }
    public decimal Balance { get; set; }
    public decimal DueInterest { get; set; }
    public decimal DueInstallment { get; set; }
    public decimal DueFine { get; set; }
    public decimal InterestRate { get; set; }
    public string IssuedOn { get; set; }
    public string MaturesOn { get; set; }
    public string Product { get; set; }
}

public class ReqLoanDetail : ReqAccount
{
    public string? CustomerCode { get; set; }
}

public class ResLoanStatement
{
    public DateTime TranDate { get; set; }
    public DateTime InterestDate { get; set; }
    public string? Reference { get; set; }
    public decimal IssueAmount { get; set; }
    public decimal Payment { get; set; }
    public decimal Principal { get; set; }
    public decimal Interest { get; set; }
    public decimal Fine { get; set; }
    public decimal Discount { get; set; }
    public decimal Balance { get; set; }
}