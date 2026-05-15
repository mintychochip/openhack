import { test, expect } from '@playwright/test';

test.describe('Admin Journey', () => {
  test('complete admin flow: login → manage users → configure judging → view analytics', async ({ page }) => {
    const timestamp = Date.now();
    const adminEmail = 'admin@test.com';
    const adminPassword = 'AdminPassword123!';
    const testUserEmail = `testuser${timestamp}@test.com`;

    // Step 1: Login as admin
    await page.goto('/login');
    await page.fill('input[name="email"]', adminEmail);
    await page.fill('input[name="password"]', adminPassword);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/\/dashboard/);

    // Step 2: Navigate to admin dashboard
    await page.click('text=Admin');
    await expect(page.locator('text=Admin Dashboard')).toBeVisible();

    // Step 3: Manage users
    await page.click('text=Users');
    await page.click('text=Add User');
    await page.fill('input[name="email"]', testUserEmail);
    await page.fill('input[name="password"]', 'UserPassword123!');
    await page.fill('input[name="name"]', 'Test User');
    await page.selectOption('select[name="role"]', 'participant');
    await page.click('button:has-text("Create")');
    await expect(page.locator('text=' + testUserEmail)).toBeVisible();

    // Step 4: Configure judging rubric
    await page.click('text=Judging');
    await page.click('text=Rubrics');
    await page.click('text=Create Rubric');
    await page.fill('input[name="name"]', `E2E Rubric ${timestamp}`);
    await page.fill('textarea[name="description"]', 'Test rubric');
    await page.click('button:has-text("Add Criterion")');
    await page.fill('input[name="criteria[0].name"]', 'Innovation');
    await page.fill('input[name="criteria[0].weight"]', '50');
    await page.click('button:has-text("Save")');
    await expect(page.locator('text=E2E Rubric')).toBeVisible();

    // Step 5: View analytics
    await page.click('text=Analytics');
    await expect(page.locator('text=Registrations')).toBeVisible();
    await expect(page.locator('text=Teams')).toBeVisible();
    await expect(page.locator('text=Projects')).toBeVisible();

    console.log('✅ Admin journey completed successfully');
  });
});
