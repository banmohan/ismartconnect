using ISmartConnect.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.Filters;

namespace ISmartConnect;

[AttributeUsage(AttributeTargets.Class | AttributeTargets.Method)]
public class ApiKeyAuthorizeAttribute : Attribute, IAuthorizationFilter
{
    public void OnAuthorization(AuthorizationFilterContext context)
    {
        var keyStore = context.HttpContext.RequestServices.GetRequiredService<IClientAccessKeyStore>();
        var requestKey = context.HttpContext.Request.Headers["Authorization"].ToString();
        var tenant = context.HttpContext.Request.Headers["tenant"].ToString();

        if (!keyStore.TryGetClientCode(requestKey, out var requestingClient) ||
            string.IsNullOrWhiteSpace(requestingClient))
        {
            context.Result = new UnauthorizedResult();
            return;
        }

        if (tenant != requestingClient)
        {
            context.Result = new BadRequestResult();
            return;
        }

        context.HttpContext.Items["tenant"] = requestingClient;
    }
}
