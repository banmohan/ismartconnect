using System.Net;
using ISmartConnect;
using ISmartConnect.Helpers;
using ISmartConnect.Module.Contracts;
using ISmartConnect.Module.Intercom;
using Microsoft.AspNetCore.Diagnostics;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddOpenApi();
// builder.Services.AddSwaggerGen(o =>
// {
//     o.MapType<DateOnly>(() => new OpenApiSchema
//     {
//         Type = JsonSchemaType.String,
//         Format = "date",
//         Example = new OpenApiString(DateTime.Today.ToString("yyyy-MM-dd")) // Optional: provides an example format
//     });
//     o.SwaggerDoc("v2", new OpenApiInfo { Title = "ISmart Connect", Version = "v1" });
// });

builder.Services.AddControllersWithViews()
    .AddJsonOptions(x => x.JsonSerializerOptions.AssignDefaultOptions());

builder.Services.AddCors(corsOptions => corsOptions.AddDefaultPolicy(policy =>
    policy.AllowAnyHeader().AllowAnyMethod().SetIsOriginAllowed(_ => true).AllowCredentials()));

builder.Services.AddSingleton<IHttpContextAccessor, HttpContextAccessor>();
builder.Services.AddScoped<IUserMeta, UserMeta>();
builder.Services.AddScoped<IMicroServiceMeta, MicroserviceMeta>();
builder.Services.AddScoped<AccountIntercomService>();


var app = builder.Build();

app.UseExceptionHandler(err =>
{
    //var isDevelopment = app.Environment.IsDevelopment();
    err.Run(async context =>
    {
        var exception = context.Features.Get<IExceptionHandlerFeature>() ??
                        throw new Exception("Something went wrong");

        context.Response.ContentType = "application/json";
        context.Response.StatusCode = 500;
        await context.Response.WriteAsJsonAsync(new
        {
            Message = exception.Error.InnerException?.Message ?? exception.Error.Message,
            ErrorCode = HttpStatusCode.InternalServerError
        });

        // Log.Write(LogEventLevel.Error, exception.Error, "Error in {Endpoint}",
        //     exception.Endpoint?.ToString() ?? "UNKNOWN");
    });
});

// if (app.Environment.IsDevelopment())
// {
app.MapOpenApi();
// }

app.UseHttpsRedirection();

app.MapGet("/", () => $"Server is running!");
app.Run();