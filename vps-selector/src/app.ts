import { renderConfigPage } from './pages/config-page';
import { renderReportPage } from './pages/report-page';
import { renderResultPage } from './pages/result-page';
import { renderTestPage } from './pages/test-page';

export type Route = 'config' | 'test' | 'result' | 'report';

export type AppState = {
  configText: string;
  reportMarkdown: string;
  errorMessage: string;
};

export type AppController = {
  state: AppState;
  setConfigText: (content: string) => void;
  setReportMarkdown: (report: string) => void;
  setErrorMessage: (message: string) => void;
  navigate: (route: Route) => void;
};

const sampleConfig = `["探测设置"]
"默认测试分钟数" = 10
"ICMP间隔毫秒" = 1000
"TCP超时毫秒" = 3000
"并发数" = 4
"白天时段" = "08:00-18:00"
"晚上时段" = "18:00-23:30"

["端口设置"]
"默认端口" = [22, 80, 443]

["总权重"]
"稳定性" = 0.50
"分时段表现" = 0.30
"性能" = 0.20

["稳定性权重"]
"可连接性" = 0.60
"丢包率" = 0.30
"连续失败" = 0.10

["分时段权重"]
"白天" = 0.60
"晚上" = 0.40

["性能权重"]
"平均延迟" = 0.40
"P95延迟" = 0.25
"抖动" = 0.20
"TCP连接耗时" = 0.10
"路由跳数" = 0.05

[["候选IP"]]
"IP" = "192.3.81.8"
"城市" = "纽约"

[["候选IP"]]
"IP" = "107.174.51.158"
"城市" = "洛杉矶 DCO3"
"端口" = [22, 443]

[["候选IP"]]
"IP" = "198.23.228.15"
"城市" = "伊利诺伊州芝加哥"
`;

export function createApp(root: HTMLElement): AppController {
  let route: Route = 'config';

  const state: AppState = {
    configText: sampleConfig,
    reportMarkdown: '',
    errorMessage: '',
  };

  const controller: AppController = {
    state,
    setConfigText(content) {
      state.configText = content;
    },
    setReportMarkdown(report) {
      state.reportMarkdown = report;
    },
    setErrorMessage(message) {
      state.errorMessage = message;
    },
    navigate(nextRoute) {
      route = nextRoute;
      render();
    },
  };

  function render() {
    root.innerHTML = '';
    root.className = 'app-shell';

    const frame = document.createElement('div');
    frame.className = 'console-frame';
    root.append(frame);

    if (route === 'config') {
      renderConfigPage(frame, controller);
      return;
    }

    if (route === 'test') {
      renderTestPage(frame, controller);
      return;
    }

    if (route === 'result') {
      renderResultPage(frame, controller);
      return;
    }

    renderReportPage(frame, controller);
  }

  render();
  return controller;
}
