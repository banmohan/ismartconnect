namespace ISmartConnect.Module.Models;

public class ReqFullStatement
{
    public string? BranchId { get; set; }
    public string? AccountNumber { get; set; }
    public DateOnly FromDate { get; set; }
    public DateOnly ToDate { get; set; }
}