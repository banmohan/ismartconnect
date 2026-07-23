namespace ISmartConnect.Models;

public class DashboardViewModel
{
    public DateTimeOffset TodayUtc { get; init; }
    public int TodayRequestCount { get; init; }
    public int TodayFailedCount { get; init; }
    public double TodaySuccessRatePct { get; init; }
    public double TodayAvgDurationMs { get; init; }

    public IReadOnlyList<string> ChartLabels { get; init; } = [];
    public IReadOnlyList<int> ChartValues { get; init; } = [];

    public IReadOnlyList<DashboardClientRow> Clients { get; init; } = [];
    public IReadOnlyList<DashboardAttentionItem> NeedsAttention { get; init; } = [];
    public IReadOnlyList<DashboardEndpointFailRow> TopFailingEndpoints { get; init; } = [];
    public IReadOnlyList<DashboardRecentError> RecentErrors { get; init; } = [];
    public IReadOnlyList<DashboardClientRow> ZeroTrafficClients { get; init; } = [];
}

public class DashboardClientRow
{
    public Guid Id { get; init; }
    public string Code { get; init; } = string.Empty;
    public string Name { get; init; } = string.Empty;
    public bool IsActive { get; init; }
    public int TodayRequestCount { get; init; }
    public int TodayFailedCount { get; init; }
}

public class DashboardAttentionItem
{
    public string ClientCode { get; init; } = string.Empty;
    public string? ClientName { get; init; }
    public string Path { get; init; } = string.Empty;
    public string? ErrorMessage { get; init; }
    public DateTimeOffset LastFailedAt { get; init; }
    public int ConsecutiveFailures { get; init; } = 5;
}

public class DashboardEndpointFailRow
{
    public string Path { get; init; } = string.Empty;
    public int FailCount { get; init; }
}

public class DashboardRecentError
{
    public Guid Id { get; init; }
    public DateTimeOffset RequestedAt { get; init; }
    public string Path { get; init; } = string.Empty;
    public string? ClientCode { get; init; }
    public int StatusCode { get; init; }
    public string? ErrorMessage { get; init; }
}
