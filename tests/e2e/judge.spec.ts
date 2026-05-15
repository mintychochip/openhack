import { test, expect } from '@playwright/test';

test.describe('Judge Journey', () => {
  test('complete judge flow: login → view assignments → score project → view history', async ({ page }) => {
    const timestamp = Date.now();
    const email = `judge${timestamp}@test.com`;
    const password = 'JudgePassword123!';

    // Step 1: Login as judge
    await page.goto('/login');
    await page.fill('input[name="email"]', email);
    await page.fill('input[name="password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL(/\/dashboard/);

    // Step 2: Navigate to judging dashboard
    await page.click('text=Judging');
    await expect(page.locator('text=Assignments')).toBeVisible();

    // Step 3: View assignments
    const assignmentCards = page.locator('[data-testid="assignment-card"]');
    const assignmentCount = await assignmentCards.count();
    console.log(`Found ${assignmentCount} assignments`);

    // Step 4: Score a project (if assignments exist)
    if (assignmentCount > 0) {
      await assignmentCards.first().click();
      await expect(page.locator('text=Score Project')).toBeVisible();

      // Fill in scores using sliders
      const sliders = page.locator('input[type="range"]');
      const sliderCount = await sliders.count();
      for (let i = 0; i < sliderCount; i++) {
        await sliders.nth(i).fill('80');
      }

      await page.fill('textarea[name="comments"]', 'Great project!');
      await page.click('button:has-text("Submit Score")');
      await expect(page.locator('text=Score submitted')).toBeVisible();
    }

    // Step 5: View score history
    await page.click('text=History');
    await expect(page.locator('text=Score History')).toBeVisible();

    console.log('✅ Judge journey completed successfully');
  });
});
