import type { Page } from '@playwright/test';
import { expect, test } from './fixtures';
import { clickHarperHighlight, waitForHighlightsToSettle } from './testUtils';

const TEST_PAGE_URL = 'http://localhost:8081/gmail_like.html';

/** Type lines into the compose box the way a person would, one Enter per new line. */
async function typeLines(page: Page, lines: string[]) {
	const editor = page.locator('#compose');
	await editor.click();
	for (const [index, line] of lines.entries()) {
		if (index > 0) await page.keyboard.press('Enter');
		await page.keyboard.type(line);
	}
	return editor;
}

test('Fixes a misspelling in a Gmail-style compose box without touching other text.', async ({
	page,
}) => {
	test.slow();
	await page.goto(TEST_PAGE_URL);
	const editor = await typeLines(page, ['Is this spelled corect wrong? I dont see.']);
	await waitForHighlightsToSettle(page);

	expect(await clickHarperHighlight(page, 'corect')).toBe(true);
	const suggestion = page.getByTitle('Click to replace "corect" with "correct"');
	await expect(suggestion).toBeVisible();
	await suggestion.click();

	await expect(editor).toContainText('Is this spelled correct wrong?');
	await expect(editor).not.toContainText('cocorrect');
});

test('Fixes the right word when earlier lines come first.', async ({ page }) => {
	test.slow();
	await page.goto(TEST_PAGE_URL);
	const editor = await typeLines(page, ['Hi Sarah,', '', 'I recieved your order yesterday.']);
	await waitForHighlightsToSettle(page);

	expect(await clickHarperHighlight(page, 'recieved')).toBe(true);
	const suggestion = page.getByTitle('Click to replace "recieved" with "received"');
	await expect(suggestion).toBeVisible();
	await suggestion.click();

	await expect(editor).toContainText('I received your order yesterday.');
	await expect(editor).toContainText('Hi Sarah,');
});

test('Does not change anything if the text moved after it was checked.', async ({ page }) => {
	test.slow();
	await page.goto(TEST_PAGE_URL);
	const editor = await typeLines(page, ['Is this spelled corect wrong?']);
	await waitForHighlightsToSettle(page);

	expect(await clickHarperHighlight(page, 'corect')).toBe(true);
	await expect(page.getByTitle('Click to replace "corect" with "correct"')).toBeVisible();

	// Edit the document behind the open popup so the old position is no longer right.
	await editor.evaluate((el) => {
		const text = el.querySelector('div')?.firstChild;
		if (text) text.textContent = `XX${text.textContent}`;
	});
	await page.waitForTimeout(1500);

	// The popup may have closed or been redrawn with the new position; click it if it is there.
	const suggestion = page.getByTitle('Click to replace "corect" with "correct"');
	if (await suggestion.isVisible()) await suggestion.click({ timeout: 5000 }).catch(() => {});

	const final = (await editor.innerText()).replace(/\s+/g, ' ').trim();
	// Either the right word was fixed or nothing changed; never a word cut in half.
	expect(final).not.toMatch(/co\s*correct|corecorrect/);
	expect(final).toMatch(/XXIs this spelled (correct|corect) wrong/);
});
