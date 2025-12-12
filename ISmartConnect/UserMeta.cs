using ISmartConnect.Module.Contracts;

namespace ISmartConnect;

public class UserMeta : IUserMeta
{
    public UserMeta(IHttpContextAccessor accessor, IConfiguration configuration)
    {
        var context = accessor.HttpContext;
        if (context == null) return;
        
        var requestKey = context.Request.Headers["Authorization"].ToString();
        ClientCode = configuration[$"AccessKeys:{requestKey}"] ?? "";
    }

    public required string ClientCode { get; set; }
}