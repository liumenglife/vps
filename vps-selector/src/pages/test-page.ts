import type { AppController } from '../app';
import { startProbe } from '../utils/tauri-api';

export function renderTestPage(root: HTMLElement, controller: AppController) {
  root.innerHTML = `
    <section class="center-panel">
      <p class="eyebrow">LIVE PROBE</p>
      <h1>测试中</h1>
      <div class="scanner" aria-hidden="true"><span></span></div>
      <p class="lede">正在调度候选 IP 探测，完成后将自动进入结果页面。</p>
      <p id="test-error" class="feedback is-error" role="alert"></p>
      <button id="back-config" type="button" class="secondary-button">返回配置</button>
    </section>
  `;

  const errorBox = root.querySelector<HTMLElement>('#test-error');
  const backButton = root.querySelector<HTMLButtonElement>('#back-config');

  backButton?.addEventListener('click', () => {
    controller.navigate('config');
  });

  void startProbe(controller.state.configText)
    .then((report) => {
      controller.setReportMarkdown(report);
      controller.setErrorMessage('');
      controller.navigate('result');
    })
    .catch((error) => {
      const message = `测试失败：${String(error)}`;
      controller.setErrorMessage(message);
      if (errorBox) {
        errorBox.textContent = message;
      }
    });
}
