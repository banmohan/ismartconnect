namespace ISmartConnect.Entities;

public class RequestLog
{
    public Guid Id { get; set; }
    public DateTimeOffset RequestedAt { get; set; }
    public int DurationMs { get; set; }
    public string HttpMethod { get; set; } = string.Empty;
    public string Path { get; set; } = string.Empty;
    public string? QueryString { get; set; }
    public Guid? ClientId { get; set; }
    public string? ClientCode { get; set; }
    public string? IpAddress { get; set; }
    public string? RequestHeaders { get; set; }
    public string? RequestBody { get; set; }
    public int StatusCode { get; set; }
    public string? ResponseBody { get; set; }
    public bool IsError { get; set; }
    public string? ErrorMessage { get; set; }
    public string? ExceptionDetails { get; set; }

    public Client? Client { get; set; }
}
