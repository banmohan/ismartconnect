using ISmartConnect.Module.Contracts;
using ISmartConnect.Services;

namespace ISmartConnect;

public class UserMeta : IUserMeta
{
    public UserMeta(IHttpContextAccessor accessor, IClientAccessKeyStore keyStore)
    {
        var context = accessor.HttpContext;
        if (context == null)
        {
            ClientCode = string.Empty;
            return;
        }

        if (context.Items.TryGetValue("tenant", out var tenant) && tenant is string code &&
            !string.IsNullOrWhiteSpace(code))
        {
            ClientCode = code;
            return;
        }

        var requestKey = context.Request.Headers["Authorization"].ToString();
        ClientCode = keyStore.TryGetClientCode(requestKey, out var clientCode)
            ? clientCode ?? string.Empty
            : string.Empty;
    }

    public required string ClientCode { get; set; }
}
