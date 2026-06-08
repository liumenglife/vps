import { expect, test } from '@playwright/test';

test('配置页展示真实候选 IP 并提供可操作入口', async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') {
      consoleErrors.push(message.text());
    }
  });

  await page.goto('/');

  await expect(page.getByRole('heading', { name: '线路选择控制台' })).toBeVisible();
  await expect(page.getByLabel('中文 TOML 配置')).toHaveValue(/192\.3\.81\.8/);
  await expect(page.getByLabel('中文 TOML 配置')).toHaveValue(/107\.174\.51\.158/);
  await expect(page.getByLabel('中文 TOML 配置')).toHaveValue(/198\.23\.228\.15/);
  await expect(page.getByRole('button', { name: '校验配置' })).toBeVisible();
  await expect(page.getByRole('button', { name: '立即开始测试' })).toBeVisible();

  await expect.poll(() => consoleErrors).toEqual([]);
});

test('测试中页面展示可解析的配置细节', async ({ page }) => {
  await page.goto('/');

  await page.getByRole('button', { name: '立即开始测试' }).click();

  await expect(page.getByRole('heading', { name: '测试中' })).toBeVisible();
  await expect(page.getByText('北京时间')).toBeVisible();
  await expect(page.getByText('判定时段')).toBeVisible();
  await expect(page.getByText('候选 IP 数量')).toBeVisible();
  await expect(page.getByText('3 个')).toBeVisible();
  await expect(page.getByText('192.3.81.8')).toBeVisible();
  await expect(page.getByText('107.174.51.158')).toBeVisible();
  await expect(page.getByText('198.23.228.15')).toBeVisible();
  await expect(page.getByText('默认端口')).toBeVisible();
  await expect(page.locator('.metric-card').filter({ hasText: '默认端口' }).getByText('22, 80, 443')).toBeVisible();
  await expect(page.getByRole('row', { name: /107\.174\.51\.158.*22, 443/ })).toBeVisible();
  await expect(page.getByText('预计测试分钟数')).toBeVisible();
  await expect(page.getByText('10 分钟')).toBeVisible();
  await expect(page.getByText('ICMP 间隔')).toBeVisible();
  await expect(page.getByText('1000 ms')).toBeVisible();
  await expect(page.getByText('TCP 超时')).toBeVisible();
  await expect(page.getByText('3000 ms')).toBeVisible();
  await expect(page.getByText('并发数')).toBeVisible();
  await expect(page.getByText('4', { exact: true })).toBeVisible();
  await expect(page.getByText('路由追踪')).toBeVisible();
  await expect(page.getByText('ICMP', { exact: true })).toBeVisible();
  await expect(page.getByText('TCP', { exact: true })).toBeVisible();
});

test('测试中页面在非白天非晚上时按北京时间展示其他时段', async ({ page }) => {
  await page.clock.setFixedTime(new Date('2026-06-08T16:00:00Z'));
  await page.goto('/');
  const config = await page.getByLabel('中文 TOML 配置').inputValue();

  await page.evaluate(async (configText) => {
    const { renderTestPage } = await import('/src/pages/test-page.ts');
    const root = document.querySelector<HTMLElement>('.app-shell');
    if (!root) {
      throw new Error('missing app root');
    }

    renderTestPage(root, {
      state: {
        configText: configText.replaceAll('[["候选IP"]]', '[[ "候选IP" ]]'),
        reportMarkdown: '',
        errorMessage: '',
      },
      setConfigText() {},
      setReportMarkdown() {},
      setErrorMessage() {},
      navigate() {},
    });
  }, config);

  await expect(page.getByRole('heading', { name: '测试中' })).toBeVisible();
  await expect(page.getByText('其他（白天 08:00-18:00 / 晚上 18:00-23:30）')).toBeVisible();
  await expect(page.getByText('3 个')).toBeVisible();
});

test('测试中页面配置解析失败时展示返回检查提示', async ({ page }) => {
  await page.goto('/');

  await page.getByLabel('中文 TOML 配置').fill('不是合法的中文 TOML 配置');
  await page.getByRole('button', { name: '立即开始测试' }).click();

  await expect(page.getByRole('heading', { name: '测试中' })).toBeVisible();
  await expect(page.getByText('配置细节解析失败，请返回配置检查')).toBeVisible();
  await expect(page.getByRole('button', { name: '返回配置' })).toBeVisible();
});

test('普通浏览器环境点击校验时展示中文失败反馈', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: '校验配置' }).click();

  await expect(page.getByText(/校验失败：/)).toBeVisible();
});

test('普通浏览器环境点击导出报告时展示中文失败反馈', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(async () => {
    const { renderReportPage } = await import('/src/pages/report-page.ts');
    const root = document.querySelector<HTMLElement>('.app-shell');
    if (!root) {
      throw new Error('missing app root');
    }

    renderReportPage(root, {
      state: {
        configText: '',
        reportMarkdown: '# VPS 测试报告\n\n有内容',
        errorMessage: '',
      },
      setConfigText() {},
      setReportMarkdown() {},
      setErrorMessage() {},
      navigate() {},
    });
  });

  await page.getByRole('button', { name: '导出 .md' }).click();

  await expect(page.getByText(/导出失败：/)).toBeVisible();
});

test('空报告点击导出时展示暂无报告反馈', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(async () => {
    const { renderReportPage } = await import('/src/pages/report-page.ts');
    const root = document.querySelector<HTMLElement>('.app-shell');
    if (!root) {
      throw new Error('missing app root');
    }

    renderReportPage(root, {
      state: {
        configText: '',
        reportMarkdown: '   ',
        errorMessage: '',
      },
      setConfigText() {},
      setReportMarkdown() {},
      setErrorMessage() {},
      navigate() {},
    });
  });

  await page.getByRole('button', { name: '导出 .md' }).click();

  await expect(page.getByText('暂无报告可导出')).toBeVisible();
});
