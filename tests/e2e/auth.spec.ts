import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('user can register, login, and logout', async ({ page }) => {
    const timestamp = Date.now();
    const email = `user${timestamp}@test.com`;
    const password = 'Password123!';

    // Register
    await page.goto('/register');
    await page.fill('input[name="email"]', email);
    await page.fill('input[name="password"]', password);
    await page.fill('input[name="name"]', 'Test User');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/\/dashboard/);

    // Logout
    await page.click('[data-testid="user-menu"]');
    await page.click('text=Logout');
    await expect(page).toHaveURL('/');

    // Login
    await page.goto('/login');
    await page.fill('input[name="email"]', email);
    await page.fill('input[name="password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/\/dashboard/);

    // Verify user info
    await page.click('[data-testid="user-menu"]');
    await expect(page.locator('text=Test User')).toBeVisible();
  });

  test('forgot password flow', async ({ page }) => {
    const email = 'test@example.com';

    await page.goto('/forgot-password');
    await page.fill('input[name="email"]', email);
    await page.click('button[type="submit"]');
    await expect(page.locator('text=Reset email sent')).toBeVisible();
  });

  test('OAuth login buttons are visible', async ({ page }) => {
    await page.goto('/login');
    
    // Check for OAuth buttons (may be disabled if not configured)
    const githubButton = page.locator('button:has-text("GitHub")');
    const googleButton = page.locator('button:has-text("Google")');
    const discordButton = page.locator('button:has-text("Discord")');
    
    // At least one OAuth button should be present
    const oauthCount = await Promise.all([
      githubButton.count(),
      googleButton.count(),
      discordButton.count(),
    ]).then(counts => counts.reduce((a, b) => a + b, 0));
    
    expect(oauthCount).toBeGreaterThan(0);
  });
});
