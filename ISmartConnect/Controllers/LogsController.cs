using ISmartConnect.Data;
using ISmartConnect.Models;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

namespace ISmartConnect.Controllers;

[Authorize]
public class LogsController(AppDbContext db) : Controller
{
    [HttpGet]
    public async Task<IActionResult> Index([FromQuery] LogFilterViewModel filter)
    {
        if (filter.Page < 1) filter.Page = 1;
        if (filter.PageSize is < 1 or > 100) filter.PageSize = 25;

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

        var total = await query.CountAsync();
        var items = await query
            .OrderByDescending(l => l.RequestedAt)
            .Skip((filter.Page - 1) * filter.PageSize)
            .Take(filter.PageSize)
            .ToListAsync();

        ViewBag.Filter = filter;
        ViewBag.ClientCodes = await db.Clients.AsNoTracking()
            .OrderBy(c => c.Code)
            .Select(c => c.Code)
            .ToListAsync();

        return View(new PagedResult<Entities.RequestLog>
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

    private static DateTimeOffset ToUtc(DateTimeOffset value) =>
        value.Offset == TimeSpan.Zero
            ? value
            : new DateTimeOffset(DateTime.SpecifyKind(value.DateTime, DateTimeKind.Utc));
}
