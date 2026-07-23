using System.ComponentModel.DataAnnotations;

namespace ISmartConnect.Models;

public class LoginViewModel
{
    [Required]
    [Display(Name = "Username")]
    public string Username { get; set; } = string.Empty;

    [Required]
    [DataType(DataType.Password)]
    public string Password { get; set; } = string.Empty;

    public string? ReturnUrl { get; set; }
}

public class UserFormViewModel
{
    public Guid? Id { get; set; }

    [Required]
    [MaxLength(100)]
    public string Username { get; set; } = string.Empty;

    [Required]
    [MaxLength(200)]
    [Display(Name = "Display name")]
    public string DisplayName { get; set; } = string.Empty;

    [DataType(DataType.Password)]
    [Display(Name = "Password")]
    public string? Password { get; set; }

    [Display(Name = "Active")]
    public bool IsActive { get; set; } = true;
}

public class ClientFormViewModel
{
    public Guid? Id { get; set; }

    [Required]
    [MaxLength(100)]
    [Display(Name = "Client code")]
    public string Code { get; set; } = string.Empty;

    [Required]
    [MaxLength(200)]
    public string Name { get; set; } = string.Empty;

    [MaxLength(200)]
    [Display(Name = "API key")]
    public string? ApiKey { get; set; }

    [Display(Name = "Active")]
    public bool IsActive { get; set; } = true;
}

public class LogFilterViewModel
{
    public DateTimeOffset? From { get; set; }
    public DateTimeOffset? To { get; set; }
    public string? Path { get; set; }
    public string? ClientCode { get; set; }
    public int? StatusCode { get; set; }
    public bool? IsError { get; set; }
    public string? Search { get; set; }
    public string SortBy { get; set; } = "requested_at";
    public string SortDir { get; set; } = "desc";
    public int Page { get; set; } = 1;
    public int PageSize { get; set; } = 25;
}

public class PagedResult<T>
{
    public IReadOnlyList<T> Items { get; init; } = [];
    public int TotalCount { get; init; }
    public int Page { get; init; }
    public int PageSize { get; init; }
    public int TotalPages => PageSize <= 0 ? 0 : (int)Math.Ceiling(TotalCount / (double)PageSize);
}
