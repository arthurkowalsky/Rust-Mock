
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

export interface Endpoint {
  method: HttpMethod;
  path: string;
  response: any;
  status?: number;
  headers?: Record<string, string>;
  proxy_url?: string;
}

export interface RequestLog {
  method: HttpMethod;
  path: string;
  request_headers: Record<string, string>;
  query: string;
  request_body?: any;

  status: number;
  response_body?: any;
  response_headers: Record<string, string>;

  timestamp: string;
  matched_endpoint?: string;
  proxied_to?: string;
}

export interface EndpointResponse {
  added?: boolean;
  removed?: boolean;
  error?: string;
}

export interface KeyValuePair {
  key: string;
  value: string;
}

export interface Tab {
  id: string;
  label: string;
  icon: React.ComponentType<any>;
}

export interface ServerConfig {
  host: string;
  port: number;
}

export interface ProxyConfig {
  proxy_url: string | null;
  enabled: boolean;
}
