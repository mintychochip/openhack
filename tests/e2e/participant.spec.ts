import { test, expect } from '@playwright/test';

test.describe('Participant Journey', () => {
  test('complete participant flow: register → create team → submit project → vote', async ({ page }) => {
    const timestamp = Date.now();
    const email = `participant${timestamp}@test.com`;
    const password = 'TestPassword123!';
    const teamName = `Test Team ${timestamp}`;
    const projectName = `Test Project ${timestamp}`;

    // Step 1: Register
    await page.goto('/');
    await page.click('text=Sign Up');
    await page.fill('input[name="email"]', email);
    await page.fill('input[name="password"]', password);
    await page.fill('input[name="name"]', 'Test Participant');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/\/dashboard/);

    // Step 2: Create team
    await page.click('text=Teams');
    await page.click('text=Create Team');
    await page.fill('input[name="name"]', teamName);
    await page.fill('textarea[name="description"]', 'Test team for E2E');
    await page.click('button[type="submit"]');
    await expect(page.locator('text=' + teamName)).toBeVisible();

    // Step 3: Submit project
    await page.click('text=Projects');
    await page.click('text=Submit Project');
    await page.fill('input[name="title"]', projectName);
    await page.fill('textarea[name="description"]', 'Test project description');
    await page.fill('input[name="repo_url"]', 'https://github.com/test/test');
    await page.click('button[type="submit"]');
    await expect(page.locator('text=' + projectName)).toBeVisible();

    // Step 4: Vote on projects
    await page.click('text=Leaderboard');
    const voteButtons = page.locator('button:has-text("Vote")');
    const voteCount = await voteButtons.count();
    if (voteCount > 0) {
      await voteButtons.first().click();
      await expect(page.locator('text=Vote recorded')).toBeVisible();
    }

    console.log('✅ Participant journey completed successfully');
  });
});
