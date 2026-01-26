namespace ISmartConnect.Module.Models;

public class ResAllAccount
{
    public IEnumerable<DepositAcc> Deposit { get; set; }
    public IEnumerable<LoanAcc> Loan { get; set; }
    public IEnumerable<ShareAcc> Share { get; set; }
}

public class DepositAcc
{
    public string MemberCode { get; set; }
    public string AccountNumber { get; set; }
    public string BranchId { get; set; }
    public string OpenDate { get; set; }
    public string Product { get; set; }
    public string AccountType { get; set; }
    public string InstFreq { get; set; }
    public decimal InstAmount { get; set; }
    public string MaturityDate { get; set; }
    public decimal IntRate { get; set; }
    public decimal AccruedInterest { get; set; }
    public decimal Balance { get; set; }
    public decimal? MinBalance { get; set; }
}

public class LoanAcc
{
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
    public string AccountType { get; set; }
}

public class ShareAcc
{
    public string MemberCode { get; set; }
    public string AccountNumber { get; set; }
    public string OpenDate { get; set; }
    public decimal Balance { get; set; }
    public string Product { get; set; }
    public string AccountType { get; set; }
    // public List<int> KittaNumber { get; set; }
}