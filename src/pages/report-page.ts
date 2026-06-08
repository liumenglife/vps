import type { AppController } from '../app';
import { exportMarkdownReport } from '../utils/report-export';

export function renderReportPage(root: HTMLElement, controller: AppController) {
  root.innerHTML = `
    <section class="hero-panel">
      <div>
        <p class="eyebrow">MARKDOWN REPORT</p>
        <h1>测试报告</h1>
        <p class="lede">预览后端生成的 Markdown 报告，可直接导出为 .md 文件。</p>
      </div>
      <div class="button-row">
        <button id="export-report" type="button" class="primary-button">导出 .md</button>
        <button id="back-result" type="button" class="secondary-button">返回结果</button>
      </div>
    </section>
    <p id="report-feedback" class="feedback" role="status"></p>
    <pre class="report-preview"><code></code></pre>
  `;

  const code = root.querySelector<HTMLElement>('.report-preview code');
  const feedback = root.querySelector<HTMLElement>('#report-feedback');
  const exportButton = root.querySelector<HTMLButtonElement>('#export-report');
  const hasReport = controller.state.reportMarkdown.trim().length > 0;

  if (code) {
    code.textContent = controller.state.reportMarkdown || '暂无报告，请先完成测试。';
  }

  root.querySelector('#back-result')?.addEventListener('click', () => controller.navigate('result'));
  exportButton?.addEventListener('click', async () => {
    if (!feedback) {
      return;
    }

    if (!hasReport) {
      feedback.textContent = '暂无报告可导出';
      feedback.className = 'feedback is-error';
      return;
    }

    exportButton.disabled = true;
    feedback.textContent = '正在导出...';
    feedback.className = 'feedback';

    try {
      const result = await exportMarkdownReport(controller.state.reportMarkdown);
      if (result.status === 'cancelled') {
        feedback.textContent = '已取消导出';
        feedback.className = 'feedback';
      } else {
        feedback.textContent = `已导出：${result.path}`;
        feedback.className = 'feedback is-success';
      }
    } catch (error) {
      feedback.textContent = `导出失败：${String(error)}`;
      feedback.className = 'feedback is-error';
    } finally {
      exportButton.disabled = false;
    }
  });
}
