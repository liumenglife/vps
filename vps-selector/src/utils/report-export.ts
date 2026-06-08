export type ExportMarkdownReportResult =
  | { status: 'cancelled' }
  | { status: 'exported'; path: string };

export async function exportMarkdownReport(markdown: string): Promise<ExportMarkdownReportResult> {
  const content = markdown.trim();
  if (!content) {
    throw new Error('没有报告内容，无法导出');
  }

  const [{ save }, { writeTextFile }] = await Promise.all([
    import('@tauri-apps/plugin-dialog'),
    import('@tauri-apps/plugin-fs'),
  ]);

  const path = await save({
    defaultPath: `vps-route-report-${new Date().toISOString().slice(0, 10)}.md`,
    filters: [{ name: 'Markdown', extensions: ['md'] }],
  });

  if (!path) {
    return { status: 'cancelled' };
  }

  await writeTextFile(path, markdown);
  return { status: 'exported', path };
}
