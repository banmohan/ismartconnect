using System.Diagnostics;
using System.Text;
using ISmartConnect.Data;
using ISmartConnect.Entities;
using Microsoft.EntityFrameworkCore;

namespace ISmartConnect.Middleware;

public class RequestResponseLoggingMiddleware(RequestDelegate next)
{
    private const int MaxBodyLength = 64 * 1024;

    private static readonly HashSet<string> SkipPrefixes = new(StringComparer.OrdinalIgnoreCase)
    {
        "/swagger",
        "/openapi",
        "/Account",
        "/Dashboard",
        "/Logs",
        "/Users",
        "/Clients",
        "/css",
        "/js",
        "/lib",
        "/favicon"
    };

    public async Task InvokeAsync(HttpContext context, IServiceScopeFactory scopeFactory)
    {
        if (!ShouldLog(context.Request.Path))
        {
            await next(context);
            return;
        }

        var requestedAt = DateTimeOffset.UtcNow;
        var stopwatch = Stopwatch.StartNew();

        context.Request.EnableBuffering();
        var requestBody = await ReadStreamAsync(context.Request.Body, MaxBodyLength);
        context.Request.Body.Position = 0;

        var originalBody = context.Response.Body;
        await using var responseBuffer = new MemoryStream();
        context.Response.Body = responseBuffer;

        Exception? caught = null;
        try
        {
            await next(context);
        }
        catch (Exception ex)
        {
            caught = ex;
            if (context.Response.StatusCode < 400)
                context.Response.StatusCode = StatusCodes.Status500InternalServerError;
            throw;
        }
        finally
        {
            stopwatch.Stop();

            responseBuffer.Position = 0;
            var responseBody = await ReadStreamAsync(responseBuffer, MaxBodyLength);
            responseBuffer.Position = 0;
            try
            {
                await responseBuffer.CopyToAsync(originalBody);
            }
            catch
            {
                // Response may already be committed by the exception handler.
            }

            context.Response.Body = originalBody;

            var tenant = context.Items.TryGetValue("tenant", out var t) ? t?.ToString() : null;
            var statusCode = caught != null && context.Response.StatusCode < 400
                ? StatusCodes.Status500InternalServerError
                : context.Response.StatusCode;
            var isError = caught != null || statusCode >= 400;
            var log = new RequestLog
            {
                Id = Guid.NewGuid(),
                RequestedAt = requestedAt,
                DurationMs = (int)Math.Min(stopwatch.ElapsedMilliseconds, int.MaxValue),
                HttpMethod = context.Request.Method,
                Path = context.Request.Path.Value ?? string.Empty,
                QueryString = context.Request.QueryString.HasValue
                    ? Truncate(context.Request.QueryString.Value, 2000)
                    : null,
                ClientCode = tenant,
                IpAddress = context.Connection.RemoteIpAddress?.ToString(),
                RequestHeaders = Truncate(FormatHeaders(context.Request.Headers), MaxBodyLength),
                RequestBody = Truncate(requestBody, MaxBodyLength),
                StatusCode = statusCode,
                ResponseBody = Truncate(responseBody, MaxBodyLength),
                IsError = isError,
                ErrorMessage = caught != null
                    ? Truncate(caught.InnerException?.Message ?? caught.Message, 2000)
                    : null,
                ExceptionDetails = caught != null ? Truncate(caught.ToString(), MaxBodyLength) : null
            };

            _ = PersistLogAsync(scopeFactory, log, tenant);
        }
    }

    private static bool ShouldLog(PathString path)
    {
        var value = path.Value ?? string.Empty;
        if (value is "/" or "")
            return false;

        foreach (var prefix in SkipPrefixes)
        {
            if (value.StartsWith(prefix, StringComparison.OrdinalIgnoreCase))
                return false;
        }

        return true;
    }

    private static async Task PersistLogAsync(IServiceScopeFactory scopeFactory, RequestLog log, string? tenant)
    {
        try
        {
            using var scope = scopeFactory.CreateScope();
            var db = scope.ServiceProvider.GetRequiredService<AppDbContext>();

            if (!string.IsNullOrWhiteSpace(tenant))
            {
                var clientId = await db.Clients.AsNoTracking()
                    .Where(c => c.Code == tenant)
                    .Select(c => (Guid?)c.Id)
                    .FirstOrDefaultAsync();
                log.ClientId = clientId;
            }

            db.RequestLogs.Add(log);
            await db.SaveChangesAsync();
        }
        catch
        {
            // Logging must never break the API pipeline.
        }
    }

    private static string FormatHeaders(IHeaderDictionary headers)
    {
        var sb = new StringBuilder();
        foreach (var header in headers)
        {
            var value = header.Key.Equals("Authorization", StringComparison.OrdinalIgnoreCase)
                ? "[REDACTED]"
                : header.Value.ToString();
            sb.Append(header.Key).Append(": ").Append(value).AppendLine();
        }

        return sb.ToString();
    }

    private static async Task<string> ReadStreamAsync(Stream stream, int maxLength)
    {
        using var reader = new StreamReader(stream, Encoding.UTF8, detectEncodingFromByteOrderMarks: false,
            bufferSize: 1024, leaveOpen: true);
        var buffer = new char[Math.Min(maxLength, 8192)];
        var sb = new StringBuilder();
        int read;
        while (sb.Length < maxLength &&
               (read = await reader.ReadAsync(buffer.AsMemory(0, Math.Min(buffer.Length, maxLength - sb.Length)))) > 0)
        {
            sb.Append(buffer, 0, read);
        }

        if (!stream.CanSeek && sb.Length >= maxLength)
            sb.Append("…[truncated]");

        return sb.ToString();
    }

    private static string? Truncate(string? value, int maxLength)
    {
        if (string.IsNullOrEmpty(value))
            return value;
        return value.Length <= maxLength ? value : value[..maxLength] + "…[truncated]";
    }
}
