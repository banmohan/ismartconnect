namespace ISmartConnect.Module.Models;

public class ResAccountValidation
{
    public string? CustomerCode { get; set; }
    public string? CustomerName { get; set; }
    public string? Address { get; set; }
    public string? MobileNumber { get; set; }
    public string? BranchId { get; set; }
    public bool AccStatus { get; set; }
    public string? DateOfBirth { get; set; }
    public string? Gender { get; set; }
    public decimal AccruedInterest { get; set; }
    public decimal InterestRate { get; set; }
    public string? AccountType { get; set; }
    public decimal AvailableBalance { get; set; }
    public decimal MinBalance { get; set; }
    public string? IdType { get; set; }
    public string? IdNumber { get; set; }
    public string? IdIssuePlace { get; set; }
    public string? IdIssueDate { get; set; }
    public string? Email { get; set; }
    public string? Occupation { get; set; }
    public string? FathersName { get; set; }
    public string? MothersName { get; set; }
    public string? GrandFathersName { get; set; }
    public string? SpouseName { get; set; }
}