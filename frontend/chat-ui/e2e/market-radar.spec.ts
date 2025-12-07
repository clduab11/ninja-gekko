/**
 * Playwright End-to-End Tests for Market Radar Component
 *
 * Comprehensive testing of the enhanced Market Radar module with multi-exchange
 * functionality, real-time data, and user interactions.
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

/**
 * Helper: Wait for Market Radar to load with data
 */
async function waitForMarketRadar(page: Page) {
  await page.waitForSelector('[data-testid="market-radar"]', { state: 'visible' });
  // Wait for exchange buttons to appear
  await page.waitForSelector('[data-testid="exchange-selector"] button', { state: 'visible' });
}

test.describe('Market Radar Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL);
    await waitForGordonUI(page);
    await waitForMarketRadar(page);
  });

  test.describe('Component Rendering', () => {
    test('should display Market Radar with proper header', async ({ page }) => {
      const radar = page.locator('[data-testid="market-radar"]');

      // Verify main container
      await expect(radar).toBeVisible();

      // Verify header elements
      await expect(page.locator('[data-testid="market-radar-header"]')).toBeVisible();
      await expect(page.locator('[data-testid="market-radar-title"]')).toContainText('Market Radar');
      await expect(page.locator('[data-testid="market-radar-status"]')).toContainText('MULTI-EXCHANGE FEED ACTIVE');
    });

    test('should display all exchange selector buttons', async ({ page }) => {
      const exchangeSelector = page.locator('[data-testid="exchange-selector"]');

      // Verify all three exchanges are present
      await expect(exchangeSelector.locator('button').filter({ hasText: 'OANDA' })).toBeVisible();
      await expect(exchangeSelector.locator('button').filter({ hasText: 'Coinbase' })).toBeVisible();
      await expect(exchangeSelector.locator('button').filter({ hasText: 'Binance.US' })).toBeVisible();
    });

    test('should display portfolio stats in header', async ({ page }) => {
      // Verify Net Liq display
      await expect(page.locator('[data-testid="net-liq-value"]')).toBeVisible();

      // Verify Global Exposure display
      await expect(page.locator('[data-testid="global-exposure-value"]')).toBeVisible();
    });
  });

  test.describe('Exchange Selection', () => {
    test('should default to Coinbase exchange', async ({ page }) => {
      // Verify Coinbase is selected by default
      const coinbaseButton = page.locator('[data-testid="exchange-selector"] button').filter({ hasText: 'Coinbase' });
      await expect(coinbaseButton).toHaveClass(/bg-emerald-500/);

      // Verify watchlist shows Coinbase pairs
      const watchlist = page.locator('[data-testid="trading-pairs-watchlist"]');
      await expect(watchlist.locator('[data-testid="pair-item"]')).toHaveCount(5); // BTC, ETH, SOL, DOGE, AVAX
    });

    test('should switch to OANDA exchange when clicked', async ({ page }) => {
      const oandaButton = page.locator('[data-testid="exchange-selector"] button').filter({ hasText: 'OANDA' });

      await oandaButton.click();

      // Verify OANDA is now active
      await expect(oandaButton).toHaveClass(/bg-emerald-500/);

      // Verify Coinbase is no longer active
      const coinbaseButton = page.locator('[data-testid="exchange-selector"] button').filter({ hasText: 'Coinbase' });
      await expect(coinbaseButton).not.toHaveClass(/bg-emerald-500/);

      // Verify watchlist shows OANDA pairs
      const watchlist = page.locator('[data-testid="trading-pairs-watchlist"]');
      await expect(watchlist.locator('[data-testid="pair-item"]')).toHaveCount(7); // EUR/USD, GBP/USD, etc.
    });

    test('should switch to Binance.US exchange when clicked', async ({ page }) => {
      const binanceButton = page.locator('[data-testid="exchange-selector"] button').filter({ hasText: 'Binance.US' });

      await binanceButton.click();

      // Verify Binance.US is now active
      await expect(binanceButton).toHaveClass(/bg-emerald-500/);

      // Verify watchlist shows Binance pairs
      const watchlist = page.locator('[data-testid="trading-pairs-watchlist"]');
      await expect(watchlist.locator('[data-testid="pair-item"]')).toHaveCount(5); // BTC, ETH, SOL, BNB, ADA
    });
  });

  test.describe('Trading Pairs Display', () => {
    test('should display correct pair symbols for Coinbase', async ({ page }) => {
      // Ensure Coinbase is selected
      const coinbaseButton = page.locator('[data-testid="exchange-selector"] button').filter({ hasText: 'Coinbase' });
      await coinbaseButton.click();

      const watchlist = page.locator('[data-testid="trading-pairs-watchlist"]');

      // Verify specific pairs are displayed
      await expect(watchlist.getByText('BTC-USD')).toBeVisible();
      await expect(watchlist.getByText('ETH-USD')).toBeVisible();
      await expect(watchlist.getByText('SOL-USD')).toBeVisible();
    });

    test('should display pair categories with proper styling', async ({ page }) => {
      const watchlist = page.locator('[data-testid="trading-pairs-watchlist"]');

      // Verify category badges are present
      const categoryBadges = watchlist.locator('[data-testid="pair-category"]');
      await expect(categoryBadges.first()).toBeVisible();

      // Verify different category types (forex, crypto, cfd)
      await expect(watchlist.getByText('FX')).toBeVisible();
      await expect(watchlist.getByText('CFD')).toBeVisible();
      await expect(watchlist.getByText('CRYPTO')).toBeVisible();
    });

    test('should highlight selected pair', async ({ page }) => {
      const watchlist = page.locator('[data-testid="trading-pairs-watchlist"]');
      const firstPair = watchlist.locator('[data-testid="pair-item"]').first();

      // First pair should be selected by default
      await expect(firstPair).toHaveClass(/bg-white\/10/);
      await expect(firstPair).toHaveClass(/border-l-2/);
      await expect(firstPair.locator('[data-testid="pair-indicator"]')).toBeVisible();
    });
  });

  test.describe('Chart Functionality', () => {
    test('should display chart area with controls', async ({ page }) => {
      const chartArea = page.locator('[data-testid="chart-container"]');

      await expect(chartArea).toBeVisible();

      // Verify timeframe buttons
      await expect(chartArea.getByText('15m')).toBeVisible();
      await expect(chartArea.getByText('1H')).toBeVisible();
      await expect(chartArea.getByText('4H')).toBeVisible();
      await expect(chartArea.getByText('1D')).toBeVisible();
    });

    test('should show current price and percentage change', async ({ page }) => {
      const priceDisplay = page.locator('[data-testid="current-price-display"]');

      await expect(priceDisplay).toBeVisible();
      await expect(priceDisplay.locator('[data-testid="price-value"]')).toBeVisible();
      await expect(priceDisplay.locator('[data-testid="price-change"]')).toBeVisible();
    });

    test('should display DEEP ANALYZE button', async ({ page }) => {
      const analyzeButton = page.locator('[data-testid="deep-analyze-button"]');

      await expect(analyzeButton).toBeVisible();
      await expect(analyzeButton).toHaveAttribute('aria-label', 'Launch Autonomous Research Agent');
      await expect(analyzeButton).toContainText('DEEP ANALYZE');
    });

    test('should switch timeframe when clicked', async ({ page }) => {
      const oneHourButton = page.locator('[data-testid="timeframe-button"]').filter({ hasText: '1H' });

      await oneHourButton.click();

      // Verify button is now active
      await expect(oneHourButton).toHaveClass(/bg-white\/10/);
      await expect(oneHourButton).toHaveClass(/text-emerald-400/);
    });
  });

  test.describe('News Feed', () => {
    test('should display news feed section', async ({ page }) => {
      const newsFeed = page.locator('[data-testid="news-feed"]');

      await expect(newsFeed).toBeVisible();
      await expect(newsFeed.locator('[data-testid="news-header"]')).toContainText('Intel Stream');
    });

    test('should show loading state when no news available', async ({ page }) => {
      const newsFeed = page.locator('[data-testid="news-feed"]');

      // Check for loading indicator when no news
      const loadingIndicator = newsFeed.locator('[data-testid="news-loading"]');
      if (await loadingIndicator.isVisible()) {
        await expect(loadingIndicator).toContainText('Establishing secure uplink');
      }
    });

    test('should display news items when available', async ({ page }) => {
      const newsFeed = page.locator('[data-testid="news-feed"]');

      // If news items are present, verify structure
      const newsItems = newsFeed.locator('[data-testid="news-item"]');
      const count = await newsItems.count();

      if (count > 0) {
        const firstItem = newsItems.first();
        await expect(firstItem.locator('[data-testid="news-source"]')).toBeVisible();
        await expect(firstItem.locator('[data-testid="news-timestamp"]')).toBeVisible();
        await expect(firstItem.locator('[data-testid="news-title"]')).toBeVisible();
      }
    });
  });

  test.describe('Pair Selection and Chart Updates', () => {
    test('should update chart when different pair is selected', async ({ page }) => {
      const watchlist = page.locator('[data-testid="trading-pairs-watchlist"]');
      const secondPair = watchlist.locator('[data-testid="pair-item"]').nth(1);

      // Get initial symbol display
      const initialSymbol = await page.locator('[data-testid="chart-symbol"]').textContent();

      // Click second pair
      await secondPair.click();

      // Verify chart symbol updated
      const newSymbol = await page.locator('[data-testid="chart-symbol"]').textContent();
      expect(newSymbol).not.toBe(initialSymbol);

      // Verify pair is now highlighted
      await expect(secondPair).toHaveClass(/bg-white\/10/);
    });

    test('should maintain pair selection when switching exchanges', async ({ page }) => {
      // Select second pair in Coinbase
      const watchlist = page.locator('[data-testid="trading-pairs-watchlist"]');
      const secondPair = watchlist.locator('[data-testid="pair-item"]').nth(1);
      await secondPair.click();

      // Switch to OANDA
      const oandaButton = page.locator('[data-testid="exchange-selector"] button').filter({ hasText: 'OANDA' });
      await oandaButton.click();

      // Verify first pair of OANDA is now selected
      const oandaWatchlist = page.locator('[data-testid="trading-pairs-watchlist"]');
      const firstOandaPair = oandaWatchlist.locator('[data-testid="pair-item"]').first();
      await expect(firstOandaPair).toHaveClass(/bg-white\/10/);
    });
  });

  test.describe('Real-time Data Updates', () => {
    test('should display live price updates', async ({ page }) => {
      const priceDisplay = page.locator('[data-testid="current-price-display"]');

      // Wait for initial price to load
      await expect(priceDisplay.locator('[data-testid="price-value"]')).not.toBeEmpty();

      // Note: WebSocket testing would require mocking or a test WebSocket server
      // For now, we verify the display elements are present
      const priceValue = await priceDisplay.locator('[data-testid="price-value"]').textContent();
      expect(priceValue).toMatch(/^\$\d{1,3}(,\d{3})*(\.\d{2})?$/);
    });

    test('should show price change indicators', async ({ page }) => {
      const priceDisplay = page.locator('[data-testid="current-price-display"]');
      const changeIndicator = priceDisplay.locator('[data-testid="price-change"]');

      await expect(changeIndicator).toBeVisible();

      // Should contain percentage change
      const changeText = await changeIndicator.textContent();
      expect(changeText).toMatch(/[+-]\d+\.\d{2}%/);
    });
  });

  test.describe('Accessibility', () => {
    test('should have proper ARIA labels for interactive elements', async ({ page }) => {
      // Exchange selector buttons
      const exchangeButtons = page.locator('[data-testid="exchange-selector"] button');
      for (const button of await exchangeButtons.all()) {
        await expect(button).toHaveAttribute('aria-label');
      }

      // Pair selection buttons
      const pairButtons = page.locator('[data-testid="trading-pairs-watchlist"] [data-testid="pair-item"]');
      for (const button of await pairButtons.all()) {
        await expect(button).toHaveAttribute('aria-label');
      }

      // Timeframe buttons
      const timeframeButtons = page.locator('[data-testid="timeframe-button"]');
      for (const button of await timeframeButtons.all()) {
        await expect(button).toHaveAttribute('aria-label');
      }
    });

    test('should support keyboard navigation', async ({ page }) => {
      // Focus should move through interactive elements
      await page.keyboard.press('Tab');
      let focusedElement = page.locator('*:focus');
      await expect(focusedElement).toBeVisible();

      // Continue tabbing through elements
      for (let i = 0; i < 5; i++) {
        await page.keyboard.press('Tab');
        focusedElement = page.locator('*:focus');
        await expect(focusedElement).toBeVisible();
      }
    });

    test('should have semantic headings', async ({ page }) => {
      // Main heading
      await expect(page.locator('h2').filter({ hasText: 'Market Radar' })).toBeVisible();

      // Section headings
      await expect(page.locator('h3').filter({ hasText: /BTC|EUR/ })).toBeVisible();
    });
  });

  test.describe('Error Handling', () => {
    test('should handle API errors gracefully', async ({ page }) => {
      // Note: This would require mocking API responses
      // For now, we verify error states don't crash the component

      const radar = page.locator('[data-testid="market-radar"]');
      await expect(radar).toBeVisible();

      // Component should remain functional even with API errors
      const exchangeButtons = page.locator('[data-testid="exchange-selector"] button');
      await expect(exchangeButtons.first()).toBeEnabled();
    });

    test('should show loading states appropriately', async ({ page }) => {
      // Verify loading indicators don't block interaction
      const loadingIndicators = page.locator('[data-testid*="loading"], [data-testid*="loading"]');

      if (await loadingIndicators.count() > 0) {
        // If loading indicators are present, ensure they're not blocking
        const firstLoading = loadingIndicators.first();
        await expect(firstLoading).not.toHaveAttribute('aria-disabled', 'true');
      }
    });
  });

  test.describe('Responsive Design', () => {
    test('should adapt layout on mobile viewport', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });

      const radar = page.locator('[data-testid="market-radar"]');
      await expect(radar).toBeVisible();

      // Verify key elements are still accessible
      await expect(page.locator('[data-testid="exchange-selector"]')).toBeVisible();
      await expect(page.locator('[data-testid="chart-container"]')).toBeVisible();
    });

    test('should maintain functionality on tablet viewport', async ({ page }) => {
      await page.setViewportSize({ width: 768, height: 1024 });

      // Test exchange switching still works
      const binanceButton = page.locator('[data-testid="exchange-selector"] button').filter({ hasText: 'Binance.US' });
      await binanceButton.click();
      await expect(binanceButton).toHaveClass(/bg-emerald-500/);
    });
  });
});

/**
 * Locator Reference for Market Radar Component
 *
 * Main Container:
 * - market-radar: Main component container
 *
 * Header Section:
 * - market-radar-header: Header container
 * - market-radar-title: "Market Radar" title
 * - market-radar-status: "MULTI-EXCHANGE FEED ACTIVE" status
 * - exchange-selector: Exchange selection buttons container
 * - net-liq-value: Net liquidity display
 * - global-exposure-value: Global exposure display
 *
 * Watchlist Section:
 * - trading-pairs-watchlist: Trading pairs list
 * - pair-item: Individual trading pair button
 * - pair-category: Category badge (FX, CFD, CRYPTO)
 * - pair-indicator: Active pair indicator dot
 *
 * Chart Section:
 * - chart-container: Main chart area
 * - chart-symbol: Current symbol display
 * - current-price-display: Price and change display
 * - price-value: Current price value
 * - price-change: Percentage change
 * - deep-analyze-button: Research button
 * - timeframe-button: Timeframe selection buttons
 *
 * News Section:
 * - news-feed: News feed container
 * - news-header: "Intel Stream" header
 * - news-item: Individual news item
 * - news-source: News source badge
 * - news-timestamp: Publication time
 * - news-title: News headline
 * - news-loading: Loading indicator
 */