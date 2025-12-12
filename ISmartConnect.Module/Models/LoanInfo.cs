namespace ISmartConnect.Module.Models;

public record AcValidationResModel
{
    public long Id { get; set; }
    public long CustomerId { get; set; }
    public string AccountName { get; set; }
    public string PAddress { get; set; }
    public string Contact { get; set; }
    public string AccountNumber { get; set; }
    public decimal InterestRate { get; set; }
    public string ProductName { get; set; }
    public bool Status { get; set; }
    public decimal MinBalance { get; set; }
}

public record LoanInfoResModel
{
    public long Id { get; set; }
    public string ProductName { get; set; }
    public string AccountNumber { get; set; }
    public decimal InterestRate { get; set; }
    public DateTime IssuedOn { get; set; }
    public DateTime MaturesOn { get; set; }
    public bool IsFlat { get; set; }
    public int InterestInsts { get; set; }
    public int PrincipalInsts { get; set; }
    public int Duration { get; set; }
}

public record LoanStmtResModel
{
    public DateTime TranDate { get; set; }
    public DateTime ValueDate { get; set; }
    public string StatementReference { get; set; }
    public decimal Issue { get; set; }
    public decimal Payment { get; set; }
    public decimal Principal { get; set; }
    public decimal Interest { get; set; }
    public decimal Fine { get; set; }
    public decimal Penalty { get; set; }
    public decimal Idiscount { get; set; }
    public decimal Pdiscount { get; set; }
}

public record LoanScheduleResModel
{
    public int ScheduleNumber { get; set; }
    public DateTime ScheduleDate { get; set; }
    public string ScheduleDateNp { get; set; }
    public decimal ScheduleAmount { get; set; }
    public decimal Principal { get; set; }
    public decimal Interest { get; set; }
    public decimal BalanceCd { get; set; }
}

public record LoanDueInfo
{
    public decimal Interest { get; set; }
    public decimal Fine { get; set; }
    public decimal Penalty { get; set; }
    public decimal Idiscount { get; set; }
    public decimal Pdiscount { get; set; }
    public decimal Principal { get; set; }
    public int CurrentDuration { get; set; }
    public DateTime LastPaid { get; set; }

    //public DateTime LastPaid { get; set; }
    public decimal PaidPrincipal { get; set; }
    public decimal DueInstallment { get; set; }
}