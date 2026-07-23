using ISmartConnect.Entities;
using Microsoft.EntityFrameworkCore;

namespace ISmartConnect.Data;

public class AppDbContext(DbContextOptions<AppDbContext> options) : DbContext(options)
{
    public DbSet<User> Users => Set<User>();
    public DbSet<Client> Clients => Set<Client>();
    public DbSet<RequestLog> RequestLogs => Set<RequestLog>();

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);

        modelBuilder.Entity<User>(entity =>
        {
            entity.ToTable("users");
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Username).HasMaxLength(100).IsRequired();
            entity.Property(e => e.PasswordHash).HasMaxLength(500).IsRequired();
            entity.Property(e => e.DisplayName).HasMaxLength(200).IsRequired();
            entity.HasIndex(e => e.Username).IsUnique().HasDatabaseName("users_username_uidx");
        });

        modelBuilder.Entity<Client>(entity =>
        {
            entity.ToTable("clients");
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Code).HasMaxLength(100).IsRequired();
            entity.Property(e => e.Name).HasMaxLength(200).IsRequired();
            entity.Property(e => e.ApiKey).HasMaxLength(200).IsRequired();
            entity.HasIndex(e => e.Code).IsUnique().HasDatabaseName("clients_code_uidx");
            entity.HasIndex(e => e.ApiKey).IsUnique().HasDatabaseName("clients_api_key_uidx");
        });

        modelBuilder.Entity<RequestLog>(entity =>
        {
            entity.ToTable("request_logs");
            entity.HasKey(e => e.Id);
            entity.Property(e => e.HttpMethod).HasMaxLength(16).IsRequired();
            entity.Property(e => e.Path).HasMaxLength(500).IsRequired();
            entity.Property(e => e.QueryString).HasMaxLength(2000);
            entity.Property(e => e.ClientCode).HasMaxLength(100);
            entity.Property(e => e.IpAddress).HasMaxLength(100);
            entity.Property(e => e.ErrorMessage).HasMaxLength(2000);

            entity.HasOne(e => e.Client)
                .WithMany(c => c.RequestLogs)
                .HasForeignKey(e => e.ClientId)
                .OnDelete(DeleteBehavior.SetNull)
                .HasConstraintName("request_logs_client_id_fkey");

            entity.HasIndex(e => e.RequestedAt)
                .HasDatabaseName("request_logs_requested_at_idx")
                .IsDescending(true);
            entity.HasIndex(e => e.Path).HasDatabaseName("request_logs_path_idx");
            entity.HasIndex(e => e.ClientCode).HasDatabaseName("request_logs_client_code_idx");
            entity.HasIndex(e => e.StatusCode).HasDatabaseName("request_logs_status_code_idx");
            entity.HasIndex(e => e.IsError).HasDatabaseName("request_logs_is_error_idx");
        });
    }
}
