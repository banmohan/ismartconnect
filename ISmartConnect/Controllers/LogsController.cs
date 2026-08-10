using ISmartConnect.Data;
using ISmartConnect.Entities;
using ISmartConnect.Models;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

namespace ISmartConnect.Controllers;

[Authorize]
public class LogsController(AppDbContext db) : Controller
{
    private static readonly HashSet<string> AllowedSortBy = new(StringComparer.OrdinalIgnoreCase)
    {
        "requested_at",
        "duration_ms",
        "status_code",
        "path",
        "client_code",
        "http_method"
    };

    [HttpGet]
    public async Task<IActionResult> Index([FromQuery] LogFilterViewModel filter)
    {
        if (filter.Page < 1) filter.Page = 1;
        if (filter.PageSize is < 1 or > 100) filter.PageSize = 25;
        if (string.IsNullOrWhiteSpace(filter.SortBy) || !AllowedSortBy.Contains(filter.SortBy))
            filter.SortBy = "requested_at";
        filter.SortDir = string.Equals(filter.SortDir, "asc", StringComparison.OrdinalIgnoreCase)
            ? "asc"
            : "desc";

        var query = db.RequestLogs.AsNoTracking().AsQueryable();

        // datetime-local binds with the server local offset; Npgsql timestamptz requires UTC.
        // Treat the picked wall-clock as UTC to match the log table display.
        if (filter.From.HasValue)
        {
            var fromUtc = ToUtc(filter.From.Value);
            query = query.Where(l => l.RequestedAt >= fromUtc);
        }

        if (filter.To.HasValue)
        {
            var toUtc = ToUtc(filter.To.Value);
            query = query.Where(l => l.RequestedAt <= toUtc);
        }

        if (!string.IsNullOrWhiteSpace(filter.Path))
            query = query.Where(l => l.Path.Contains(filter.Path));
        if (!string.IsNullOrWhiteSpace(filter.ClientCode))
            query = query.Where(l => l.ClientCode == filter.ClientCode);
        if (filter.StatusCode.HasValue)
            query = query.Where(l => l.StatusCode == filter.StatusCode.Value);
        if (filter.IsError.HasValue)
            query = query.Where(l => l.IsError == filter.IsError.Value);
        if (!string.IsNullOrWhiteSpace(filter.Search))
        {
            var term = filter.Search;
            query = query.Where(l =>
                (l.RequestBody != null && l.RequestBody.Contains(term)) ||
                (l.ResponseBody != null && l.ResponseBody.Contains(term)) ||
                (l.ErrorMessage != null && l.ErrorMessage.Contains(term)));
        }

        query = ApplySort(query, filter.SortBy, filter.SortDir);

        var total = await query.CountAsync();
        var items = await query
            .Skip((filter.Page - 1) * filter.PageSize)
            .Take(filter.PageSize)
            .ToListAsync();

        ViewBag.Filter = filter;
        ViewBag.ClientCodes = await db.Clients.AsNoTracking()
            .OrderBy(c => c.Code)
            .Select(c => c.Code)
            .ToListAsync();

        return View(new PagedResult<RequestLog>
        {
            Items = items,
            TotalCount = total,
            Page = filter.Page,
            PageSize = filter.PageSize
        });
    }

    [HttpGet]
    public async Task<IActionResult> Details(Guid id)
    {
        var log = await db.RequestLogs.AsNoTracking().FirstOrDefaultAsync(l => l.Id == id);
        if (log is null)
            return NotFound();

        return View(log);
    }

    private static IQueryable<RequestLog> ApplySort(IQueryable<RequestLog> query, string sortBy, string sortDir)
    {
        var asc = sortDir == "asc";
        return sortBy.ToLowerInvariant() switch
        {
            "duration_ms" => asc ? query.OrderBy(l => l.DurationMs) : query.OrderByDescending(l => l.DurationMs),
            "status_code" => asc ? query.OrderBy(l => l.StatusCode) : query.OrderByDescending(l => l.StatusCode),
            "path" => asc ? query.OrderBy(l => l.Path) : query.OrderByDescending(l => l.Path),
            "client_code" => asc
                ? query.OrderBy(l => l.ClientCode)
                : query.OrderByDescending(l => l.ClientCode),
            "http_method" => asc
                ? query.OrderBy(l => l.HttpMethod)
                : query.OrderByDescending(l => l.HttpMethod),
            _ => asc
                ? query.OrderBy(l => l.RequestedAt)
                : query.OrderByDescending(l => l.RequestedAt)
        };
    }

    private static DateTimeOffset ToUtc(DateTimeOffset value) =>
        value.Offset == TimeSpan.Zero
            ? value
            : new DateTimeOffset(DateTime.SpecifyKind(value.DateTime, DateTimeKind.Utc));
}
