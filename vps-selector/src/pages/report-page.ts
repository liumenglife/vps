import type { AppController } from '../app';

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
    <pre class="report-preview"><code></code></pre>
  `;

  const code = root.querySelector<HTMLElement>('.report-preview code');
  if (code) {
    code.textContent = controller.state.reportMarkdown || '暂无报告，请先完成测试。';
  }

  root.querySelector('#back-result')?.addEventListener('click', () => controller.navigate('result'));
  root.querySelector('#export-report')?.addEventListener('click', () => {
    const blob = new Blob([controller.state.reportMarkdown], { type: 'text/markdown;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `vps-route-report-${new Date().toISOString().slice(0, 10)}.md`;
    link.click();
    URL.revokeObjectURL(url);
  });
}
