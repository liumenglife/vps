import type { AppController } from '../app';
import { validateConfigText } from '../utils/tauri-api';

export function renderConfigPage(root: HTMLElement, controller: AppController) {
  root.innerHTML = `
    <section class="hero-panel">
      <div>
        <p class="eyebrow">VPS ROUTE CONTROL</p>
        <h1>线路选择控制台</h1>
        <p class="lede">输入中文 TOML 配置，校验权重和候选 IP 后即可启动 ICMP、TCP、traceroute 组合测试。</p>
      </div>
      <div class="status-card">
        <span class="status-dot"></span>
        <strong>配置待命</strong>
        <small>本地 Tauri 命令通道</small>
      </div>
    </section>
    <section class="workspace-grid">
      <label class="editor-panel">
        <span>中文 TOML 配置</span>
        <textarea id="config-editor" spellcheck="false"></textarea>
      </label>
      <aside class="side-panel">
        <h2>操作序列</h2>
        <p>先执行配置校验；校验通过后可立即开始测试。测试期间请保持网络连接稳定。</p>
        <div class="button-stack">
          <button id="validate-config" type="button" class="secondary-button">校验配置</button>
          <button id="start-test" type="button" class="primary-button">立即开始测试</button>
        </div>
        <p id="config-feedback" class="feedback" role="status"></p>
      </aside>
    </section>
  `;

  const editor = root.querySelector<HTMLTextAreaElement>('#config-editor');
  const feedback = root.querySelector<HTMLElement>('#config-feedback');
  const validateButton = root.querySelector<HTMLButtonElement>('#validate-config');
  const startButton = root.querySelector<HTMLButtonElement>('#start-test');

  if (!editor || !feedback || !validateButton || !startButton) {
    return;
  }

  editor.value = controller.state.configText;
  feedback.textContent = controller.state.errorMessage;
  feedback.className = controller.state.errorMessage ? 'feedback is-error' : 'feedback';

  editor.addEventListener('input', () => {
    controller.setConfigText(editor.value);
    controller.setErrorMessage('');
    feedback.textContent = '';
    feedback.className = 'feedback';
  });

  validateButton.addEventListener('click', async () => {
    validateButton.disabled = true;
    feedback.textContent = '正在校验配置...';
    feedback.className = 'feedback';
    try {
      const message = await validateConfigText(editor.value);
      controller.setConfigText(editor.value);
      controller.setErrorMessage('');
      feedback.textContent = message;
      feedback.className = 'feedback is-success';
    } catch (error) {
      const message = `配置校验失败：${String(error)}`;
      controller.setErrorMessage(message);
      feedback.textContent = message;
      feedback.className = 'feedback is-error';
    } finally {
      validateButton.disabled = false;
    }
  });

  startButton.addEventListener('click', () => {
    controller.setConfigText(editor.value);
    controller.setErrorMessage('');
    controller.navigate('test');
  });
}
