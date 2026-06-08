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

type ProbeLogEvent = {
  payload: string;
};

type TestPageDeps = {
  listenProbeLog: (handler: (event: ProbeLogEvent) => void) => Promise<() => void>;
  startProbe: (content: string) => Promise<string>;
};

const defaultTestPageDeps: TestPageDeps = {
  async listenProbeLog(handler) {
    const { listen } = await import('@tauri-apps/api/event');
    return listen<string>('probe-log', handler);
  },
  startProbe,
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
    </section>
  `;
}

async function subscribeProbeLog(logBox: HTMLElement, listenProbeLog: TestPageDeps['listenProbeLog']) {
  try {
    const unlisten = await listenProbeLog((event) => {
      appendProbeLog(logBox, String(event.payload));
    });

    logBox.textContent = '';
    return unlisten;
  } catch {
    logBox.textContent = '等待 Tauri 探测日志通道';
    return undefined;
  }
}

function appendProbeLog(logBox: HTMLElement, message: string) {
  const line = document.createElement('div');
  line.textContent = message;
  logBox.append(line);
  logBox.scrollTop = logBox.scrollHeight;
}

export function renderTestPage(root: HTMLElement, controller: AppController, deps: TestPageDeps = defaultTestPageDeps) {
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
      <section class="detail-card probe-log-card" aria-label="实时探测日志">
        <h2>实时探测日志</h2>
        <div id="probe-log" class="probe-log" role="log" aria-live="polite">等待 Tauri 探测日志通道</div>
      </section>
      <p id="test-error" class="feedback is-error" role="alert"></p>
      <button id="back-config" type="button" class="secondary-button">返回配置</button>
    </section>
  `;

  const errorBox = root.querySelector<HTMLElement>('#test-error');
  const backButton = root.querySelector<HTMLButtonElement>('#back-config');
  const logBox = root.querySelector<HTMLElement>('#probe-log');
  let disposed = false;
  let unlistenProbeLog: (() => void) | undefined;

  const cleanupProbeLog = () => {
    if (!unlistenProbeLog) {
      return;
    }

    const unlisten = unlistenProbeLog;
    unlistenProbeLog = undefined;
    unlisten();
  };

  backButton?.addEventListener('click', () => {
    controller.navigate('config');
  });

  void (async () => {
    if (logBox) {
      unlistenProbeLog = await subscribeProbeLog(logBox, deps.listenProbeLog);
    }

    if (disposed) {
      cleanupProbeLog();
      return;
    }

    return deps.startProbe(controller.state.configText);
  })()
    .then((report) => {
      if (!report || disposed) {
        return;
      }

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
    })
    .finally(() => {
      cleanupProbeLog();
    });

  return () => {
    disposed = true;
    cleanupProbeLog();
  };
}
