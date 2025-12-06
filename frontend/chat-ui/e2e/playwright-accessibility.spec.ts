/**
 * Playwright Accessibility Tests for Ninja Gekko Chat UI
 * 
 * These tests demonstrate reliable automation patterns using the 
 * data-testid attributes and ARIA labels instrumented in the UI components.
 */

import { test, expect, Page } from '@playwright/test';

// Base URL for the development server
const BASE_URL = 'http://localhost:5173';

/**
 * Helper: Wait for the Gordon chat UI to fully load
 */
async function waitForGordonUI(page: Page) {
  await page.waitForSelector('[data-testid="app-container"]', { state: 'visible' });
  await page.waitForSelector('[data-testid="chat-conversation"]', { state: 'visible' });
  await page.waitForSelector('[data-testid="chat-input-field"]', { state: 'visible' });
}

test.describe('Chat Interface', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await waitForGordonUI(page);
  });

  test('should load the Gordon chat interface', async ({ page }) => {
    // Verify header elements
    await expect(page.locator('[data-testid="indicator-persona"]')).toBeVisible();
    await expect(page.locator('[data-testid="indicator-autonomous-mode"]')).toBeVisible();
    await expect(page.locator('[data-testid="indicator-mcp-status"]')).toBeVisible();
    
    // Verify main content structure
    await expect(page.locator('[data-testid="main-content"]')).toBeVisible();
    await expect(page.locator('[data-testid="chat-panel"]')).toBeVisible();
    await expect(page.locator('[data-testid="side-panels"]')).toBeVisible();
  });

  test('can send a message to Gordon', async ({ page }) => {
    // Type a trading instruction
    const chatInput = page.locator('[data-testid="chat-input-field"]');
    await expect(chatInput).toBeVisible();
    
    await chatInput.fill('Buy 0.5 BTC at market');
    
    // Click send button
    const sendButton = page.locator('[data-testid="btn-send-message"]');
    await expect(sendButton).toBeEnabled();
    await sendButton.click();
    
    // Verify user message appears in conversation
    await expect(page.locator('[data-testid="chat-message-user"]').last()).toBeVisible();
    await expect(page.locator('[data-testid="chat-message-user"]').last()).toContainText('Buy 0.5 BTC');
    
    // Wait for Gordon's response (assistant message)
    await expect(page.locator('[data-testid="chat-message-assistant"]')).toBeVisible({ timeout: 10000 });
  });

  test('can attach files to message', async ({ page }) => {
    const attachButton = page.locator('[data-testid="btn-attach-file"]');
    await expect(attachButton).toBeVisible();
    await expect(attachButton).toHaveAttribute('aria-label', 'Attach files (CSV, PDF, MD)');
  });
});

test.describe('Persona Controls', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await waitForGordonUI(page);
  });

  test('should display persona controls', async ({ page }) => {
    await expect(page.locator('[data-testid="persona-controls"]')).toBeVisible();
  });

  test('can change persona settings', async ({ page }) => {
    // Select tone
    const toneSelector = page.locator('[data-testid="select-persona-tone"]');
    await expect(toneSelector).toBeVisible();
    await toneSelector.selectOption('dramatic');
    
    // Select style
    const styleSelector = page.locator('[data-testid="select-persona-style"]');
    await styleSelector.selectOption('witty');
    
    // Select mood
    const moodSelector = page.locator('[data-testid="select-persona-mood"]');
    await moodSelector.selectOption('calm');
    
    // Update persona
    const updateButton = page.locator('[data-testid="btn-update-persona"]');
    await updateButton.click();
    
    // Verify persona indicator updates
    await expect(page.locator('[data-testid="indicator-persona"]')).toContainText('dramatic');
  });
});

test.describe('Action Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await waitForGordonUI(page);
  });

  test('should display control buttons', async ({ page }) => {
    const dashboard = page.locator('[data-testid="action-dashboard"]');
    await expect(dashboard).toBeVisible();
    
    // Verify control buttons
    await expect(page.locator('[data-testid="btn-pause-trading"]')).toBeVisible();
    await expect(page.locator('[data-testid="btn-summon-swarm"]')).toBeVisible();
    await expect(page.locator('[data-testid="btn-research-pulse"]')).toBeVisible();
  });

  test('can pause trading', async ({ page }) => {
    const pauseButton = page.locator('[data-testid="btn-pause-trading"]');
    await expect(pauseButton).toHaveAttribute('aria-label', 'Pause trading for 6 hours across all exchanges');
    
    await pauseButton.click();
    // Add assertion for expected behavior after pause
  });

  test('can summon AI swarm', async ({ page }) => {
    const swarmButton = page.locator('[data-testid="btn-summon-swarm"]');
    await expect(swarmButton).toHaveAttribute('aria-label', 'Summon AI research swarm for strategy diagnostics');
    
    await swarmButton.click();
    // Add assertion for expected behavior after swarm summon
  });
});

test.describe('Diagnostics Panel', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await waitForGordonUI(page);
  });

  test('should display diagnostics when available', async ({ page }) => {
    // Diagnostics panel may not be visible if no diagnostics
    const panel = page.locator('[data-testid="diagnostics-panel"]');
    
    // If visible, verify structure
    const isVisible = await panel.isVisible();
    if (isVisible) {
      await expect(page.locator('[data-testid="diagnostics-list"]')).toBeVisible();
    }
  });
});

test.describe('Insights Panel', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await waitForGordonUI(page);
  });

  test('should display insight tiles', async ({ page }) => {
    const insightsPanel = page.locator('[data-testid="insights-panel"]');
    await expect(insightsPanel).toBeVisible();
    
    // Verify insight tiles
    await expect(page.locator('[data-testid="insight-tile-exposure"]')).toBeVisible();
    await expect(page.locator('[data-testid="insight-tile-automation"]')).toBeVisible();
    await expect(page.locator('[data-testid="insight-tile-data-streams"]')).toBeVisible();
    await expect(page.locator('[data-testid="insight-tile-security"]')).toBeVisible();
  });

  test('can read metric values', async ({ page }) => {
    // Read exposure value
    const exposureValue = await page.locator('[data-testid="insight-tile-exposure-value"]').textContent();
    console.log(`Current exposure: ${exposureValue}`);
    
    // Read automation value
    const automationValue = await page.locator('[data-testid="insight-tile-automation-value"]').textContent();
    expect(automationValue).toContain('Swarm');
  });

  test('should display market radar', async ({ page }) => {
    const radar = page.locator('[data-testid="market-radar"]');
    await expect(radar).toBeVisible();
    
    // Wait for news items to load
    await page.waitForSelector('[data-testid^="radar-item-"]', { timeout: 10000 });
  });
});

test.describe('Trading Controls', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await waitForGordonUI(page);
  });

  test('can control pause duration slider', async ({ page }) => {
    const slider = page.locator('[data-testid="slider-pause-duration"]');
    await expect(slider).toBeVisible();
    
    // Verify ARIA attributes
    await expect(slider).toHaveAttribute('aria-valuemin', '1');
    await expect(slider).toHaveAttribute('aria-valuemax', '24');
    
    // Get current value
    const currentValue = await slider.getAttribute('aria-valuenow');
    console.log(`Current pause duration: ${currentValue} hours`);
  });

  test('can resume trading', async ({ page }) => {
    const resumeButton = page.locator('[data-testid="btn-resume-trading"]');
    await expect(resumeButton).toBeVisible();
    await expect(resumeButton).toHaveAttribute('aria-label', 'Resume trading operations');
  });
});

test.describe('Accessibility Compliance', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await waitForGordonUI(page);
  });

  test('all interactive elements have accessible labels', async ({ page }) => {
    // Chat input
    await expect(page.locator('[data-testid="chat-input-field"]'))
      .toHaveAttribute('aria-label', 'Type your message to Gordon');
    
    // Send button
    await expect(page.locator('[data-testid="btn-send-message"]'))
      .toHaveAttribute('aria-label', 'Send message');
    
    // Persona selectors
    await expect(page.locator('[data-testid="select-persona-tone"]'))
      .toHaveAttribute('aria-label', 'Select persona tone');
    await expect(page.locator('[data-testid="select-persona-style"]'))
      .toHaveAttribute('aria-label', 'Select persona style');
    await expect(page.locator('[data-testid="select-persona-mood"]'))
      .toHaveAttribute('aria-label', 'Select persona mood');
  });

  test('semantic roles are properly defined', async ({ page }) => {
    // Chat conversation has log role
    await expect(page.locator('[data-testid="chat-conversation"]')).toHaveAttribute('role', 'log');
    
    // Main content has main role
    await expect(page.locator('[data-testid="main-content"]')).toHaveAttribute('role', 'main');
    
    // Insights panel has region role
    await expect(page.locator('[data-testid="insights-panel"]')).toHaveAttribute('role', 'region');
  });

  test('aria-live regions are configured for dynamic content', async ({ page }) => {
    // Chat conversation should announce new messages
    await expect(page.locator('[data-testid="chat-conversation"]'))
      .toHaveAttribute('aria-live', 'polite');
    
    // Diagnostics list should announce updates
    const diagnosticsPanel = page.locator('[data-testid="diagnostics-panel"]');
    if (await diagnosticsPanel.isVisible()) {
      await expect(page.locator('[data-testid="diagnostics-list"]'))
        .toHaveAttribute('aria-live', 'polite');
    }
  });
});

/**
 * Locator Reference for Playwright MCP Server
 * 
 * Core Chat Elements:
 * - chat-input-field: Main message input textarea
 * - btn-send-message: Send message button
 * - chat-conversation: Message history container (role="log", aria-live="polite")
 * - chat-message-user: User messages
 * - chat-message-assistant: Gordon's responses
 * 
 * Control Buttons:
 * - btn-pause-trading: Pause trading across all exchanges
 * - btn-resume-trading: Resume trading operations
 * - btn-summon-swarm: Summon AI research swarm
 * - btn-research-pulse: Request deep research analysis
 * 
 * Persona Controls:
 * - select-persona-tone: Tone selector (concise/balanced/dramatic)
 * - select-persona-style: Style selector (analytical/witty/direct)
 * - select-persona-mood: Mood selector (direct/witty/calm)
 * - btn-update-persona: Submit persona changes
 * 
 * Metrics & Diagnostics:
 * - metric-total-equity: Account total equity display
 * - metric-net-exposure: Net exposure percentage
 * - diagnostics-panel: System diagnostics region
 * - diagnostic-item-{index}: Individual diagnostic alerts
 * 
 * Insights:
 * - insight-tile-exposure: Current exposure tile
 * - insight-tile-automation: Automation status tile
 * - insight-tile-data-streams: Data streams tile
 * - insight-tile-security: Security mode tile
 * - market-radar: Market news radar section
 * 
 * Status Indicators:
 * - indicator-persona: Current persona display
 * - indicator-loading: Loading spinner
 * - indicator-autonomous-mode: Autonomous trading mode status
 * - indicator-mcp-status: MCP Mesh connection status
 */
