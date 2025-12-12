using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.Filters;

namespace ISmartConnect;

[AttributeUsage(AttributeTargets.Class | AttributeTargets.Method)]
public class ApiKeyAuthorizeAttribute : Attribute, IAuthorizationFilter
{
    public void OnAuthorization(AuthorizationFilterContext context)
    {
        var configuration = context.HttpContext.RequestServices.GetService<IConfiguration>()!;
        var requestKey = context.HttpContext.Request.Headers["Authorization"].ToString();
        var tenant = context.HttpContext.Request.Headers["tenant"].ToString();
        var requestingClient = configuration[$"AccessKeys:{requestKey}"];
        
        if (string.IsNullOrWhiteSpace(requestingClient))
        {
            context.Result = new UnauthorizedResult();
            return;
        }

        if (tenant != requestingClient)
        {
            context.Result = new BadRequestResult();
            return;
        }
        
        context.HttpContext.Items.Add("tenant", requestingClient);
    }
}