export interface ModelDescriptor {
  id: string;
  display_name: string;
  context_window: number;
}

export interface ProviderInfo {
  id: string;
  display_name: string;
  models: ModelDescriptor[];
}

export interface LlmResponse {
  content: string;
  input_tokens: number;
  output_tokens: number;
}
