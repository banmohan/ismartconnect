using ISmartConnect.Data;
using ISmartConnect.Entities;
using ISmartConnect.Models;
using ISmartConnect.Services;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

namespace ISmartConnect.Controllers;

[Authorize]
public class ClientsController(AppDbContext db, IClientAccessKeyStore keyStore) : Controller
{
    [HttpGet]
    public async Task<IActionResult> Index()
    {
        var clients = await db.Clients.AsNoTracking()
            .OrderBy(c => c.Code)
            .ToListAsync();
        return View(clients);
    }

    [HttpGet]
    public IActionResult Create() => View("Form", new ClientFormViewModel
    {
        ApiKey = DbSeeder.GenerateApiKey(),
        IsActive = true
    });

    [HttpPost]
    [ValidateAntiForgeryToken]
    public async Task<IActionResult> Create(ClientFormViewModel model)
    {
        if (string.IsNullOrWhiteSpace(model.ApiKey))
            model.ApiKey = DbSeeder.GenerateApiKey();

        if (await db.Clients.AnyAsync(c => c.Code == model.Code))
            ModelState.AddModelError(nameof(model.Code), "Client code already exists.");
        if (await db.Clients.AnyAsync(c => c.ApiKey == model.ApiKey))
            ModelState.AddModelError(nameof(model.ApiKey), "API key already exists.");

        if (!ModelState.IsValid)
            return View("Form", model);

        var now = DateTimeOffset.UtcNow;
        db.Clients.Add(new Client
        {
            Id = Guid.NewGuid(),
            Code = model.Code.Trim(),
            Name = model.Name.Trim(),
            ApiKey = model.ApiKey.Trim(),
            IsActive = model.IsActive,
            CreatedAt = now,
            UpdatedAt = now
        });
        await db.SaveChangesAsync();
        await keyStore.RefreshAsync();

        TempData["Success"] = "Client created. Access key cache refreshed.";
        return RedirectToAction(nameof(Index));
    }

    [HttpGet]
    public async Task<IActionResult> Edit(Guid id)
    {
        var client = await db.Clients.FindAsync(id);
        if (client is null)
            return NotFound();

        return View("Form", new ClientFormViewModel
        {
            Id = client.Id,
            Code = client.Code,
            Name = client.Name,
            ApiKey = client.ApiKey,
            IsActive = client.IsActive
        });
    }

    [HttpPost]
    [ValidateAntiForgeryToken]
    public async Task<IActionResult> Edit(Guid id, ClientFormViewModel model)
    {
        var client = await db.Clients.FindAsync(id);
        if (client is null)
            return NotFound();

        if (string.IsNullOrWhiteSpace(model.ApiKey))
            ModelState.AddModelError(nameof(model.ApiKey), "API key is required.");

        if (await db.Clients.AnyAsync(c => c.Code == model.Code && c.Id != id))
            ModelState.AddModelError(nameof(model.Code), "Client code already exists.");
        if (await db.Clients.AnyAsync(c => c.ApiKey == model.ApiKey && c.Id != id))
            ModelState.AddModelError(nameof(model.ApiKey), "API key already exists.");

        if (!ModelState.IsValid)
        {
            model.Id = id;
            return View("Form", model);
        }

        client.Code = model.Code.Trim();
        client.Name = model.Name.Trim();
        client.ApiKey = model.ApiKey!.Trim();
        client.IsActive = model.IsActive;
        client.UpdatedAt = DateTimeOffset.UtcNow;

        await db.SaveChangesAsync();
        await keyStore.RefreshAsync();

        TempData["Success"] = "Client updated. Access key cache refreshed.";
        return RedirectToAction(nameof(Index));
    }
}
