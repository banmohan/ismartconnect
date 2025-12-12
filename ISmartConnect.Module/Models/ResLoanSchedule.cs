namespace ISmartConnect.Module.Models;

public class ResLoanSchedule
{
    public int ScheduleNo { get; set; }
    public DateTime ScheduleDate { get; set; }
    public string? ScheduleDateNp { get; set; }
    public decimal Amount { get; set; }
    public decimal Principal { get; set; }
    public decimal Interest { get; set; }
    public decimal PrincipalBalance { get; set; }
}