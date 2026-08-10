using System.Collections.Concurrent;
using ISmartConnect.Data;
using Microsoft.EntityFrameworkCore;

namespace ISmartConnect.Services;

public interface IClientAccessKeyStore
{
    bool TryGetClientCode(string apiKey, out string? clientCode);
    Task RefreshAsync(CancellationToken cancellationToken = default);
}

public class ClientAccessKeyStore(IServiceScopeFactory scopeFactory) : IClientAccessKeyStore
{
    private volatile ConcurrentDictionary<string, string> _keys =
        new(StringComparer.Ordinal);

    public bool TryGetClientCode(string apiKey, out string? clientCode)
    {
        if (_keys.TryGetValue(apiKey, out var code))
        {
            clientCode = code;
            return true;
        }

        clientCode = null;
        return false;
    }

    public async Task RefreshAsync(CancellationToken cancellationToken = default)
    {
        using var scope = scopeFactory.CreateScope();
        var db = scope.ServiceProvider.GetRequiredService<AppDbContext>();

        var activeClients = await db.Clients
            .AsNoTracking()
            .Where(c => c.IsActive)
            .Select(c => new { c.ApiKey, c.Code })
            .ToListAsync(cancellationToken);

        var next = new ConcurrentDictionary<string, string>(StringComparer.Ordinal);
        foreach (var client in activeClients)
            next[client.ApiKey] = client.Code;

        _keys = next;
    }
}
