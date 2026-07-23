using ISmartConnect.Data;
using ISmartConnect.Entities;
using ISmartConnect.Models;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

namespace ISmartConnect.Controllers;

[Authorize]
public class UsersController(AppDbContext db, IPasswordHasher<User> passwordHasher) : Controller
{
    [HttpGet]
    public async Task<IActionResult> Index()
    {
        var users = await db.Users.AsNoTracking()
            .OrderBy(u => u.Username)
            .ToListAsync();
        return View(users);
    }

    [HttpGet]
    public IActionResult Create() => View("Form", new UserFormViewModel());

    [HttpPost]
    [ValidateAntiForgeryToken]
    public async Task<IActionResult> Create(UserFormViewModel model)
    {
        if (string.IsNullOrWhiteSpace(model.Password))
            ModelState.AddModelError(nameof(model.Password), "Password is required.");

        if (await db.Users.AnyAsync(u => u.Username == model.Username))
            ModelState.AddModelError(nameof(model.Username), "Username already exists.");

        if (!ModelState.IsValid)
            return View("Form", model);

        var now = DateTimeOffset.UtcNow;
        var user = new User
        {
            Id = Guid.NewGuid(),
            Username = model.Username.Trim(),
            DisplayName = model.DisplayName.Trim(),
            IsActive = model.IsActive,
            CreatedAt = now,
            UpdatedAt = now
        };
        user.PasswordHash = passwordHasher.HashPassword(user, model.Password!);
        db.Users.Add(user);
        await db.SaveChangesAsync();

        TempData["Success"] = "User created.";
        return RedirectToAction(nameof(Index));
    }

    [HttpGet]
    public async Task<IActionResult> Edit(Guid id)
    {
        var user = await db.Users.FindAsync(id);
        if (user is null)
            return NotFound();

        return View("Form", new UserFormViewModel
        {
            Id = user.Id,
            Username = user.Username,
            DisplayName = user.DisplayName,
            IsActive = user.IsActive
        });
    }

    [HttpPost]
    [ValidateAntiForgeryToken]
    public async Task<IActionResult> Edit(Guid id, UserFormViewModel model)
    {
        var user = await db.Users.FindAsync(id);
        if (user is null)
            return NotFound();

        if (await db.Users.AnyAsync(u => u.Username == model.Username && u.Id != id))
            ModelState.AddModelError(nameof(model.Username), "Username already exists.");

        if (!ModelState.IsValid)
        {
            model.Id = id;
            return View("Form", model);
        }

        user.Username = model.Username.Trim();
        user.DisplayName = model.DisplayName.Trim();
        user.IsActive = model.IsActive;
        user.UpdatedAt = DateTimeOffset.UtcNow;

        if (!string.IsNullOrWhiteSpace(model.Password))
            user.PasswordHash = passwordHasher.HashPassword(user, model.Password);

        await db.SaveChangesAsync();
        TempData["Success"] = "User updated.";
        return RedirectToAction(nameof(Index));
    }
}
