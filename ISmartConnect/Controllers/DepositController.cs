using ISmartConnect.Module.Intercom;
using ISmartConnect.Module.Models;
using Microsoft.AspNetCore.Mvc;

namespace ISmartConnect.Controllers;

[ApiKeyAuthorize]
[ApiController]
[Route("api/deposit")]
public class DepositController(AccountIntercomService accountIntercom) : ControllerBase
{
    [HttpPost("balance")]
    public async Task<ActionResult<ResBalance>> Balance(ReqAccount req)
    {
        var res = await accountIntercom.BalanceEnquiryAsync(req);
        return Ok(new
        {
            res,
            IsoResponseCode = "00"
        });
    }

    [HttpPost("mini-statement")]
    public async Task<ActionResult<ResMiniStatement>> MiniStatement(ReqAccount req)
    {
        var res = await accountIntercom.MiniStatementAsync(req);
        return Ok(new
        {
            res,
            IsoResponseCode = "00"
        });
    }

    [HttpPost("full-statement")]
    public async Task<ActionResult<IEnumerable<StatementItem>>> FullStatement(ReqFullStatement req)
    {
        var res = await accountIntercom.FullStatementAsync(req);
        return Ok(new
        {
            res,
            IsoResponseCode = "00"
        });
    }

    [HttpPost("fund-transfer")]
    public async Task<ActionResult<ResFundTransfer>> FundTransfer(ReqFundTransfer req)
    {
        var res = await accountIntercom.FundTransferAsync(req);
        return Ok(new
        {
            res,
            IsoResponseCode = "00"
        });
    }

    [HttpPost("reversal")]
    public async Task<ActionResult<bool>> Balance(ResFundTransfer req)
    {
        var res = await accountIntercom.ReversalAsync(req);
        return Ok(new
        {
            res,
            IsoResponseCode = "00"
        });
    }

    [HttpPost("acc-validation")]
    public async Task<ActionResult<ResAccountValidation>> AccValidation(ReqAccount req)
    {
        var res = await accountIntercom.AccValidateAsync(req);
        return Ok(new
        {
            res,
            IsoResponseCode = "00"
        });
    }

    [HttpGet("all-account/{account}")]
    public async Task<ActionResult<ResAllAccount>> AllAccount(string account)
    {
        var res = await accountIntercom.AllAccountAsync(account);
        return Ok(new
        {
            res,
            IsoResponseCode = "00"
        });
    }
}