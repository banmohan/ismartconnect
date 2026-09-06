using System.Net;
using ISmartConnect;
using ISmartConnect.Data;
using ISmartConnect.Entities;
using ISmartConnect.Helpers;
using ISmartConnect.Middleware;
using ISmartConnect.Module.Contracts;
using ISmartConnect.Module.Intercom;
using ISmartConnect.Services;
using Microsoft.AspNetCore.Authentication.Cookies;
using Microsoft.AspNetCore.Diagnostics;
using Microsoft.AspNetCore.Identity;
using Microsoft.EntityFrameworkCore;
using Microsoft.OpenApi;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddOpenApi();
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen(o =>
{
    o.SwaggerDoc("v2", new OpenApiInfo { Title = "ISmart Connect", Version = "v2" });
});

builder.Services.AddHttpContextAccessor();
builder.Services.AddControllersWithViews()
    .AddJsonOptions(x => x.JsonSerializerOptions.AssignDefaultOptions());

builder.Services.AddCors(corsOptions => corsOptions.AddDefaultPolicy(policy =>
    policy.AllowAnyHeader().AllowAnyMethod().SetIsOriginAllowed(_ => true).AllowCredentials()));

builder.Services.AddDbContext<AppDbContext>(options =>
    options.UseNpgsql(builder.Configuration.GetConnectionString("DefaultConnection"))
        .UseSnakeCaseNamingConvention());

builder.Services.AddAuthentication(CookieAuthenticationDefaults.AuthenticationScheme)
    .AddCookie(options =>
    {
        options.LoginPath = "/Account/Login";
        options.LogoutPath = "/Account/Logout";
        options.AccessDeniedPath = "/Account/Login";
        options.SlidingExpiration = true;
        options.ExpireTimeSpan = TimeSpan.FromHours(8);
    });
builder.Services.AddAuthorization();

builder.Services.AddSingleton<IClientAccessKeyStore, ClientAccessKeyStore>();
builder.Services.AddSingleton<IPasswordHasher<User>, PasswordHasher<User>>();
builder.Services.AddScoped<IUserMeta, UserMeta>();
builder.Services.AddScoped<IMicroServiceMeta, MicroserviceMeta>();
builder.Services.AddScoped<AccountIntercomService>();

var app = builder.Build();

await DbSeeder.MigrateAndSeedAsync(app.Services);

app.UseExceptionHandler(err =>
{
    err.Run(async context =>
    {
        var exception = context.Features.Get<IExceptionHandlerFeature>() ??
                        throw new Exception("Something went wrong");

        context.Response.ContentType = "application/json";
        context.Response.StatusCode = 500;
        var isoResponseCode = "96";
        switch (exception.Error.Message)
        {
            case "InvalidAccount":
                isoResponseCode = "76";
                break;
            case "Branch is closed for EOD processing, Contact admin":
                isoResponseCode = "39";
                break;
            case "Day close process running":
                isoResponseCode = "39";
                break;
            case var msg when msg.Contains("EOD started", StringComparison.OrdinalIgnoreCase):
                isoResponseCode = "39";
                break;
            case "InvalidTransaction":
                isoResponseCode = "30";
                break;
            case "TranNotAllowed":
                isoResponseCode = "39";
                break;
            case "InsufficientFund":
                isoResponseCode = "51";
                break;
            case "UnableToProcess":
                isoResponseCode = "05";
                break;
            case "AccountDormant":
                isoResponseCode = "52";
                break;
            case "AccountClosed":
                isoResponseCode = "54";
                break;
            case "AccountRestricted":
                isoResponseCode = "62";
                break;
            case "DuplicateReversal":
                isoResponseCode = "98";
                break;
        }

        await context.Response.WriteAsJsonAsync(new
        {
            Message = exception.Error.InnerException?.Message ?? exception.Error.Message,
            ErrorCode = HttpStatusCode.InternalServerError,
            isoResponseCode
        });
    });
});

app.UseSwagger();
app.UseSwaggerUI(o => { o.SwaggerEndpoint("v2/swagger.json", "Akash Api"); });
app.MapOpenApi();

app.UseStaticFiles();
app.UseRouting();
app.UseCors();
app.UseAuthentication();
app.UseAuthorization();
app.UseMiddleware<RequestResponseLoggingMiddleware>();

app.MapControllerRoute(
    name: "default",
    pattern: "{controller=Dashboard}/{action=Index}/{id?}");

app.MapControllers();
app.Run();