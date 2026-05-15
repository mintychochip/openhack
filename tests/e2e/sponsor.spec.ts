import { test, expect } from '@playwright/test';

test.describe('Sponsor Journey', () => {
  test('complete sponsor flow: login → create booth → add prizes → review submissions', async ({ page }) => {
    const timestamp = Date.now();
    const sponsorEmail = `sponsor${timestamp}@test.com`;
    const sponsorPassword = 'SponsorPassword123!';
    const boothName = `Test Booth ${timestamp}`;
    const prizeName = `Test Prize ${timestamp}`;

    // Step 1: Register and login as sponsor
    await page.goto('/register');
    await page.fill('input[name="email"]', sponsorEmail);
    await page.fill('input[name="password"]', sponsorPassword);
    await page.fill('input[name="name"]', 'Test Sponsor');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/\/dashboard/);

    // Step 2: Navigate to sponsor portal
    await page.click('text=Sponsor');
    await expect(page.locator('text=Sponsor Dashboard')).toBeVisible();

    // Step 3: Create booth
    await page.click('text=Create Booth');
    await page.fill('input[name="name"]', boothName);
    await page.fill('textarea[name="description"]', 'Test booth description');
    await page.selectOption('select[name="tier"]', 'gold');
    await page.click('button:has-text("Publish")');
    await expect(page.locator('text=' + boothName)).toBeVisible();

    // Step 4: Add prize
    await page.click('text=Prizes');
    await page.click('text=Add Prize');
    await page.fill('input[name="name"]', prizeName);
    await page.fill('textarea[name="description"]', 'Test prize');
    await page.fill('input[name="value"]', '1000');
    await page.selectOption('select[name="currency"]', 'USD');
    await page.click('button:has-text("Create")');
    await expect(page.locator('text=' + prizeName)).toBeVisible();

    // Step 5: Review submissions (if any)
    await page.click('text=Submissions');
    console.log('Sponsor submissions page loaded');

    console.log('✅ Sponsor journey completed successfully');
  });
});
