using ISmartConnect.Data;
using ISmartConnect.Models;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

namespace ISmartConnect.Controllers;

[Authorize]
public class DashboardController(AppDbContext db) : Controller
{
    [HttpGet]
    public async Task<IActionResult> Index()
    {
        var todayStart = new DateTimeOffset(DateTime.UtcNow.Date, TimeSpan.Zero);
        var todayEnd = todayStart.AddDays(1);
        var recentActivitySince = todayStart.AddDays(-7);

        var todayLogs = db.RequestLogs.AsNoTracking()
            .Where(l => l.RequestedAt >= todayStart && l.RequestedAt < todayEnd);

        var todayRequestCount = await todayLogs.CountAsync();
        var todayFailedCount = await todayLogs.CountAsync(l => l.IsError);
        var todayAvgDurationMs = todayRequestCount == 0
            ? 0
            : await todayLogs.AverageAsync(l => (double)l.DurationMs);
        var todaySuccessRatePct = todayRequestCount == 0
            ? 100
            : Math.Round((todayRequestCount - todayFailedCount) * 100.0 / todayRequestCount, 1);

        var todayByClient = await todayLogs
            .Where(l => l.ClientCode != null && l.ClientCode != "")
            .GroupBy(l => l.ClientCode!)
            .Select(g => new
            {
                Code = g.Key,
                Total = g.Count(),
                Failed = g.Count(x => x.IsError)
            })
            .ToListAsync();

        var todayByClientMap = todayByClient.ToDictionary(x => x.Code, StringComparer.Ordinal);

        var clients = await db.Clients.AsNoTracking()
            .OrderBy(c => c.Code)
            .ToListAsync();

        var clientRows = clients.Select(c =>
        {
            todayByClientMap.TryGetValue(c.Code, out var stats);
            return new DashboardClientRow
            {
                Id = c.Id,
                Code = c.Code,
                Name = c.Name,
                IsActive = c.IsActive,
                TodayRequestCount = stats?.Total ?? 0,
                TodayFailedCount = stats?.Failed ?? 0
            };
        }).ToList();

        var chartSource = todayByClient
            .OrderByDescending(x => x.Total)
            .ToList();

        var topFailingEndpoints = await todayLogs
            .Where(l => l.IsError)
            .GroupBy(l => l.Path)
            .Select(g => new DashboardEndpointFailRow
            {
                Path = g.Key,
                FailCount = g.Count()
            })
            .OrderByDescending(x => x.FailCount)
            .Take(5)
            .ToListAsync();

        var recentErrors = await db.RequestLogs.AsNoTracking()
            .Where(l => l.IsError)
            .OrderByDescending(l => l.RequestedAt)
            .Take(10)
            .Select(l => new DashboardRecentError
            {
                Id = l.Id,
                RequestedAt = l.RequestedAt,
                Path = l.Path,
                ClientCode = l.ClientCode,
                StatusCode = l.StatusCode,
                ErrorMessage = l.ErrorMessage
            })
            .ToListAsync();

        var needsAttention = await BuildNeedsAttentionAsync(clients, recentActivitySince);

        var model = new DashboardViewModel
        {
            TodayUtc = todayStart,
            TodayRequestCount = todayRequestCount,
            TodayFailedCount = todayFailedCount,
            TodaySuccessRatePct = todaySuccessRatePct,
            TodayAvgDurationMs = Math.Round(todayAvgDurationMs, 1),
            ChartLabels = chartSource.Select(x => x.Code).ToList(),
            ChartValues = chartSource.Select(x => x.Total).ToList(),
            Clients = clientRows,
            NeedsAttention = needsAttention,
            TopFailingEndpoints = topFailingEndpoints,
            RecentErrors = recentErrors,
            ZeroTrafficClients = clientRows
                .Where(c => c.IsActive && c.TodayRequestCount == 0)
                .ToList()
        };

        return View(model);
    }

    private async Task<IReadOnlyList<DashboardAttentionItem>> BuildNeedsAttentionAsync(
        List<Entities.Client> clients,
        DateTimeOffset recentActivitySince)
    {
        var activeCodes = clients.Where(c => c.IsActive).Select(c => c.Code).ToList();
        if (activeCodes.Count == 0)
            return [];

        var recentClientCodes = await db.RequestLogs.AsNoTracking()
            .Where(l => l.RequestedAt >= recentActivitySince
                        && l.ClientCode != null
                        && activeCodes.Contains(l.ClientCode))
            .Select(l => l.ClientCode!)
            .Distinct()
            .ToListAsync();

        var nameByCode = clients.ToDictionary(c => c.Code, c => c.Name, StringComparer.Ordinal);
        var items = new List<DashboardAttentionItem>();

        foreach (var code in recentClientCodes)
        {
            var lastFive = await db.RequestLogs.AsNoTracking()
                .Where(l => l.ClientCode == code)
                .OrderByDescending(l => l.RequestedAt)
                .Take(5)
                .Select(l => new { l.IsError, l.Path, l.ErrorMessage, l.RequestedAt })
                .ToListAsync();

            if (lastFive.Count < 5 || lastFive.Any(l => !l.IsError))
                continue;

            var latest = lastFive[0];
            items.Add(new DashboardAttentionItem
            {
                ClientCode = code,
                ClientName = nameByCode.GetValueOrDefault(code),
                Path = latest.Path,
                ErrorMessage = latest.ErrorMessage,
                LastFailedAt = latest.RequestedAt,
                ConsecutiveFailures = 5
            });
        }

        return items
            .OrderByDescending(i => i.LastFailedAt)
            .ToList();
    }
}
