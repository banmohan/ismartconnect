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
        return Ok(new
        {
            res.BranchId,
            res.DueInterest,
            res.IssuedOn,
            res.DueInstallment,
            res.DueFine,
            res.Balance,
            res.DisbursedAmt,
            res.AccountNumber,
            res.InterestRate,
            res.MaturesOn,
            res.MemberCode,
            res.Product,
            IsoResponseCode = "00"
        });
    }

    [HttpPost("loan-statement")]
    public async Task<ActionResult<IEnumerable<ResLoanStatement>>> LoanStatement(ReqLoanStatement req)
    {
        var res = await accountIntercom.LoanStatementAsync(req);
        return Ok(new
        {
            StatementList = res,
            IsoResponseCode = "00"
        });
    }

    [HttpPost("loan-schedule")]
    public async Task<ActionResult<IEnumerable<ResLoanSchedule>>> LoanSchedule(ReqAccount req)
    {
        var res = await accountIntercom.LoanScheduleAsync(req);
        return Ok(new
        {
            Schedules = res,
            IsoResponseCode = "00"
        });
    }
}