using ISmartConnect.Module.Intercom;
using ISmartConnect.Module.Models;
using Microsoft.AspNetCore.Mvc;

namespace ISmartConnect.Controllers;

[ApiKeyAuthorize]
[ApiController]
[Route("api/loan")]
public class LoanController(AccountIntercomService accountIntercom) : ControllerBase
{
    [HttpPost("loan-detail")]
    public async Task<ActionResult<ResLoanDetails>> LoanDetail(ReqLoanDetail req)
    {
        var res = await accountIntercom.LoanDetailAsync(req);
        return Ok(res);
    }

    [HttpPost("loan-statement")]
    public async Task<ActionResult<IEnumerable<ResLoanStatement>>> LoanStatement(ReqLoanStatement req)
    {
        var res = await accountIntercom.LoanStatementAsync(req);
        return Ok(res);
    }

    [HttpPost("loan-schedule")]
    public async Task<ActionResult<IEnumerable<ResLoanSchedule>>> LoanSchedule(ReqAccount req)
    {
        var res = await accountIntercom.LoanScheduleAsync(req);
        return Ok(res);
    }
}