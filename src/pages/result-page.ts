import type { AppController } from '../app';

type RankingRow = {
  rank: string;
  ip: string;
  city: string;
  totalScore: string;
  stabilityScore: string;
  timePeriodScore: string;
  performanceScore: string;
  confidence: string;
};

type Recommendation = {
  ip: string;
  city: string;
};

export function parseRankingRows(markdown: string): RankingRow[] {
  const rankingSection = markdown.split(/\n##\s+排名\s*\n/)[1]?.split(/\n##\s+/)[0] ?? markdown;
  return rankingSection
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.startsWith('|') && !/^\|\s*-+/.test(line) && !line.includes('排名'))
    .map((line) => line.split('|').slice(1, -1).map((cell) => cell.trim()))
    .filter((cells) => cells.length >= 8)
    .map((cells) => ({
      rank: cells[0] ?? '',
      ip: cells[1] ?? '',
      city: cells[2] ?? '',
      totalScore: cells[3] ?? '',
      stabilityScore: cells[4] ?? '',
      timePeriodScore: cells[5] ?? '',
      performanceScore: cells[6] ?? '',
      confidence: cells[7] ?? '',
    }))
    .filter((row) => row.ip.length > 0 && row.city.length > 0);
}

export function parseRecommendation(markdown: string): Recommendation | undefined {
  const recommendationSection = markdown.split(/\n##\s+推荐结论\s*\n/)[1]?.split(/\n##\s+/)[0] ?? '';
  const match = recommendationSection.match(/推荐选择\s+([^（\s]+)（([^）]+)），总分/);
  if (!match) {
    return undefined;
  }

  return {
    ip: match[1] ?? '',
    city: match[2] ?? '',
  };
}

export function renderResultPage(root: HTMLElement, controller: AppController) {
  const rows = parseRankingRows(controller.state.reportMarkdown);
  const recommended = parseRecommendation(controller.state.reportMarkdown) ?? rows[0];

  root.innerHTML = `
    <section class="hero-panel">
      <div>
        <p class="eyebrow">PROBE RESULT</p>
        <h1>线路排行榜</h1>
        <p class="lede">按后端 Markdown 报告中的推荐结论和排名表解析展示，解析失败时回退到排名首位。</p>
      </div>
      <div class="status-card accent-card">
        <span>推荐购买</span>
        <strong>${escapeHtml(recommended?.ip ?? '暂无结果')}</strong>
        <small>${escapeHtml(recommended?.city ?? '请查看报告')}</small>
      </div>
    </section>
    <section class="result-layout">
      <div class="ranking-panel">
        <h2>候选 IP 排行榜</h2>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>排名</th>
                <th>IP</th>
                <th>城市</th>
                <th>总分</th>
                <th>稳定性</th>
                <th>分时段</th>
                <th>性能</th>
                <th>可信度</th>
              </tr>
            </thead>
            <tbody>
              ${rows.map(renderRankingRow).join('') || '<tr><td colspan="8">未从报告中解析到排名表，请查看完整报告。</td></tr>'}
            </tbody>
          </table>
        </div>
      </div>
      <aside class="side-panel">
        <h2>控制</h2>
        <p>推荐购买 IP：<strong>${escapeHtml(recommended?.ip ?? '暂无')}</strong></p>
        <p>推荐城市：<strong>${escapeHtml(recommended?.city ?? '暂无')}</strong></p>
        <div class="button-stack">
          <button id="view-report" type="button" class="primary-button">查看报告</button>
          <button id="retry-test" type="button" class="secondary-button">重新测试</button>
          <button id="back-config" type="button" class="secondary-button">返回配置</button>
        </div>
      </aside>
    </section>
  `;

  root.querySelector('#view-report')?.addEventListener('click', () => controller.navigate('report'));
  root.querySelector('#retry-test')?.addEventListener('click', () => controller.navigate('test'));
  root.querySelector('#back-config')?.addEventListener('click', () => controller.navigate('config'));
}

function renderRankingRow(row: RankingRow) {
  return `
    <tr>
      <td>${escapeHtml(row.rank)}</td>
      <td><code>${escapeHtml(row.ip)}</code></td>
      <td>${escapeHtml(row.city)}</td>
      <td>${escapeHtml(row.totalScore)}</td>
      <td>${escapeHtml(row.stabilityScore)}</td>
      <td>${escapeHtml(row.timePeriodScore)}</td>
      <td>${escapeHtml(row.performanceScore)}</td>
      <td>${escapeHtml(row.confidence)}</td>
    </tr>
  `;
}

function escapeHtml(value: string) {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}
