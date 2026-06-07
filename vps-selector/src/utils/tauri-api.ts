import { invoke } from '@tauri-apps/api/core';

export function validateConfigText(content: string): Promise<string> {
  return invoke<string>('validate_config_text', { content });
}

export function startProbe(content: string): Promise<string> {
  return invoke<string>('start_probe', { content });
}
