
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

export interface Endpoint {
  method: HttpMethod;
  path: string;
  response: any;
  status?: number;
  headers?: Record<string, string>;
}

export interface RequestLog {
  timestamp: string;
  method: HttpMethod;
  path: string;
  status: number;
  headers: Record<string, string>;
  query?: Record<string, string>;
  body?: any;
  duration?: number;
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
