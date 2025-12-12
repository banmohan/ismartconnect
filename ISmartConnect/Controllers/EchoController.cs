using ISmartConnect.Module.Intercom;
using Microsoft.AspNetCore.Mvc;

namespace ISmartConnect.Controllers;

[ApiKeyAuthorize]
[ApiController]
[Route("api/echo")]
public class EchoController(AccountIntercomService accountIntercom) : ControllerBase
{
    [HttpGet("echo")]
    public async Task<IActionResult> Echo()
    {
        var res = await accountIntercom.EchoAsync();
        if (res)
            return Ok(new
            {
                Result = "success"
            });

        return StatusCode(500);
    }
}