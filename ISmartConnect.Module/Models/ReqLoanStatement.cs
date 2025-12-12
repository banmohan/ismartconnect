namespace ISmartConnect.Module.Models;

public class ReqLoanStatement : ReqLoanDetail
{
    public DateTime FromDate { get; set; }
    public DateTime ToDate { get; set; }
}