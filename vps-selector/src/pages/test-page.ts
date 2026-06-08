import type { AppController } from '../app';
import { startProbe } from '../utils/tauri-api';

type CandidateIp = {
  ip: string;
  city: string;
  ports: number[];
};

type TestConfigDetails = {
  beijingTime: string;
  timeWindow: string;
  candidateCount: number;
  defaultPorts: number[];
  testMinutes: number;
  icmpIntervalMs: number;
  tcpTimeoutMs: number;
  concurrency: number;
  candidates: CandidateIp[];
};

function parseChineseTomlDetails(configText: string): TestConfigDetails {
  const defaultPorts = readNumberArray(configText, '默认端口');
  const dayWindow = readStringValue(configText, '白天时段');
  const nightWindow = readStringValue(configText, '晚上时段');
  const candidates = readCandidates(configText, defaultPorts);

  if (!defaultPorts.length || !dayWindow || !nightWindow || !candidates.length) {
    throw new Error('missing required config details');
  }

  const beijingNow = new Date(Date.now() + 8 * 60 * 60 * 1000);
  const beijingTime = `${beijingNow.getUTCFullYear()}-${pad2(beijingNow.getUTCMonth() + 1)}-${pad2(beijingNow.getUTCDate())} ${pad2(beijingNow.getUTCHours())}:${pad2(beijingNow.getUTCMinutes())}:${pad2(beijingNow.getUTCSeconds())}`;
  const currentMinutes = beijingNow.getUTCHours() * 60 + beijingNow.getUTCMinutes();
  const activePeriod = readActivePeriod(currentMinutes, dayWindow, nightWindow);

  return {
    beijingTime,
    timeWindow: `${activePeriod}（白天 ${dayWindow} / 晚上 ${nightWindow}）`,
    candidateCount: candidates.length,
    defaultPorts,
    testMinutes: readNumberValue(configText, '默认测试分钟数'),
    icmpIntervalMs: readNumberValue(configText, 'ICMP间隔毫秒'),
    tcpTimeoutMs: readNumberValue(configText, 'TCP超时毫秒'),
    concurrency: readNumberValue(configText, '并发数'),
    candidates,
  };
}

function readNumberValue(text: string, key: string) {
  const match = text.match(new RegExp(`"${escapeRegExp(key)}"\\s*=\\s*(\\d+)`));
  if (!match) {
    throw new Error(`missing ${key}`);
  }

  return Number(match[1]);
}

function readStringValue(text: string, key: string) {
  const match = text.match(new RegExp(`"${escapeRegExp(key)}"\\s*=\\s*"([^"]+)"`));
  return match?.[1] ?? '';
}

function readNumberArray(text: string, key: string) {
  const match = text.match(new RegExp(`"${escapeRegExp(key)}"\\s*=\\s*\\[([^\\]]*)\\]`));
  if (!match) {
    return [];
  }

  return match[1]
    .split(',')
    .map((item) => Number(item.trim()))
    .filter((port) => Number.isInteger(port) && port > 0);
}

function readCandidates(text: string, defaultPorts: number[]) {
  return text
    .split(/\[\[\s*"候选IP"\s*\]\]/)
    .slice(1)
    .map((block) => {
      const ip = readStringValue(block, 'IP');
      const city = readStringValue(block, '城市');
      const candidatePorts = readNumberArray(block, '端口');

      return {
        ip,
        city,
        ports: candidatePorts.length ? candidatePorts : defaultPorts,
      };
    })
    .filter((candidate) => candidate.ip && candidate.city);
}

function readActivePeriod(currentMinutes: number, dayWindow: string, nightWindow: string) {
  if (isWithinWindow(currentMinutes, dayWindow)) {
    return '白天';
  }

  if (isWithinWindow(currentMinutes, nightWindow)) {
    return '晚上';
  }

  return '其他';
}

function isWithinWindow(currentMinutes: number, windowText: string) {
  const [startText, endText] = windowText.split('-');
  const start = parseClockMinutes(startText);
  const end = parseClockMinutes(endText);

  if (start <= end) {
    return currentMinutes >= start && currentMinutes < end;
  }

  return currentMinutes >= start || currentMinutes < end;
}

function parseClockMinutes(value = '') {
  const match = value.trim().match(/^(\d{2}):(\d{2})$/);
  if (!match) {
    throw new Error('invalid time window');
  }

  return Number(match[1]) * 60 + Number(match[2]);
}

function pad2(value: number) {
  return String(value).padStart(2, '0');
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function escapeHtml(value: string) {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function renderDetails(details: TestConfigDetails) {
  const metrics = [
    ['北京时间', details.beijingTime],
    ['判定时段', details.timeWindow],
    ['候选 IP 数量', `${details.candidateCount} 个`],
    ['默认端口', details.defaultPorts.join(', ')],
    ['预计测试分钟数', `${details.testMinutes} 分钟`],
    ['ICMP 间隔', `${details.icmpIntervalMs} ms`],
    ['TCP 超时', `${details.tcpTimeoutMs} ms`],
    ['并发数', `${details.concurrency}`],
  ];

  return `
    <section class="probe-details" aria-label="测试配置细节">
      <div class="metric-grid">
        ${metrics
          .map(
            ([label, value]) => `
              <article class="metric-card">
                <span>${escapeHtml(label)}</span>
                <strong>${escapeHtml(value)}</strong>
              </article>
            `,
          )
          .join('')}
      </div>
      <section class="detail-card">
        <h2>候选 IP 列表</h2>
        <div class="table-wrap">
          <table>
            <thead>
              <tr><th>IP</th><th>城市</th><th>有效端口</th></tr>
            </thead>
            <tbody>
              ${details.candidates
                .map(
                  (candidate) => `
                    <tr>
                      <td><code>${escapeHtml(candidate.ip)}</code></td>
                      <td>${escapeHtml(candidate.city)}</td>
                      <td>${escapeHtml(candidate.ports.join(', '))}</td>
                    </tr>
                  `,
                )
                .join('')}
            </tbody>
          </table>
        </div>
      </section>
      <section class="detail-card">
        <h2>阶段说明</h2>
        <ol class="phase-list">
          <li><strong>解析配置</strong><span>读取中文 TOML，识别时段、端口、候选 IP 和测试参数。</span></li>
          <li><strong>路由追踪</strong><span>采集候选线路跳数，作为评分报告的一部分。</span></li>
          <li><strong>ICMP</strong><span>按配置间隔持续探测延迟、丢包和抖动。</span></li>
          <li><strong>TCP</strong><span>按有效端口尝试连接并记录连接耗时与可用性。</span></li>
          <li><strong>评分报告</strong><span>当前版本按后端一次性任务执行，完成后自动进入结果页。</span></li>
        </ol>
      </section>
    </section>
  `;
}

export function renderTestPage(root: HTMLElement, controller: AppController) {
  let detailsHtml = '<p class="feedback is-error" role="alert">配置细节解析失败，请返回配置检查</p>';

  try {
    detailsHtml = renderDetails(parseChineseTomlDetails(controller.state.configText));
  } catch {
    controller.setErrorMessage('配置细节解析失败，请返回配置检查');
  }

  root.innerHTML = `
    <section class="test-layout">
      <header class="test-hero">
        <div>
          <p class="eyebrow">LIVE PROBE</p>
          <h1>测试中</h1>
          <p class="lede">正在调度候选 IP 探测。当前版本按后端一次性任务执行，完成后将自动进入结果页面。</p>
        </div>
        <div class="scanner" aria-hidden="true"><span></span></div>
      </header>
      ${detailsHtml}
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
