namespace ISmartConnect.Module.Models;

public class ReqLoanStatement : ReqLoanDetail
{
    public DateOnly FromDate { get; set; }
    public DateOnly ToDate { get; set; }
}