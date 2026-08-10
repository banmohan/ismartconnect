using System.Security.Cryptography;
using ISmartConnect.Entities;
using ISmartConnect.Services;
using Microsoft.AspNetCore.Identity;
using Microsoft.EntityFrameworkCore;

namespace ISmartConnect.Data;

public static class DbSeeder
{
    public const string DefaultAdminUsername = "admin";
    public const string DefaultAdminPassword = "Admin@123";

    public static async Task MigrateAndSeedAsync(IServiceProvider services)
    {
        using var scope = services.CreateScope();
        var db = scope.ServiceProvider.GetRequiredService<AppDbContext>();
        var configuration = scope.ServiceProvider.GetRequiredService<IConfiguration>();
        var keyStore = scope.ServiceProvider.GetRequiredService<IClientAccessKeyStore>();
        var hasher = scope.ServiceProvider.GetRequiredService<IPasswordHasher<User>>();
        var logger = scope.ServiceProvider.GetRequiredService<ILoggerFactory>().CreateLogger("DbMigrations");

        var pending = (await db.Database.GetPendingMigrationsAsync()).ToList();
        if (pending.Count == 0)
        {
            logger.LogInformation("Database is up to date; no pending migrations.");
        }
        else
        {
            logger.LogInformation("Applying {Count} pending migration(s): {Migrations}",
                pending.Count, string.Join(", ", pending));
        }

        await db.Database.MigrateAsync();
        logger.LogInformation("Migration check complete.");

        await SeedAdminAsync(db, hasher);
        await SeedClientsFromAccessKeysAsync(db, configuration);
        await keyStore.RefreshAsync();
    }

    private static async Task SeedAdminAsync(AppDbContext db, IPasswordHasher<User> hasher)
    {
        if (await db.Users.AnyAsync())
            return;

        var now = DateTimeOffset.UtcNow;
        var admin = new User
        {
            Id = Guid.NewGuid(),
            Username = DefaultAdminUsername,
            DisplayName = "Administrator",
            IsActive = true,
            CreatedAt = now,
            UpdatedAt = now
        };
        admin.PasswordHash = hasher.HashPassword(admin, DefaultAdminPassword);
        db.Users.Add(admin);
        await db.SaveChangesAsync();
    }

    private static async Task SeedClientsFromAccessKeysAsync(AppDbContext db, IConfiguration configuration)
    {
        var accessKeys = configuration.GetSection("AccessKeys").GetChildren().ToList();
        if (accessKeys.Count == 0)
            return;

        var now = DateTimeOffset.UtcNow;
        foreach (var entry in accessKeys)
        {
            var apiKey = entry.Key;
            var code = entry.Value;
            if (string.IsNullOrWhiteSpace(apiKey) || string.IsNullOrWhiteSpace(code))
                continue;

            var exists = await db.Clients.AnyAsync(c => c.ApiKey == apiKey || c.Code == code);
            if (exists)
                continue;

            db.Clients.Add(new Client
            {
                Id = Guid.NewGuid(),
                Code = code,
                Name = code,
                ApiKey = apiKey,
                IsActive = true,
                CreatedAt = now,
                UpdatedAt = now
            });
        }

        await db.SaveChangesAsync();
    }

    public static string GenerateApiKey(int byteLength = 24)
    {
        var bytes = RandomNumberGenerator.GetBytes(byteLength);
        return Convert.ToBase64String(bytes)
            .TrimEnd('=')
            .Replace('+', 'x')
            .Replace('/', 'y');
    }
}
