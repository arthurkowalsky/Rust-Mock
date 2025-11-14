
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

export interface Endpoint {
  method: HttpMethod;
  path: string;
  response: any;
  status?: number;
  headers?: Record<string, string>;
}

export interface RequestLog {
  // Request data
  method: HttpMethod;
  path: string;
  request_headers: Record<string, string>;
  query: string;
  request_body?: any;

  // Response data
  status: number;
  response_body?: any;
  response_headers: Record<string, string>;

  // Metadata
  timestamp: string;
  matched_endpoint?: string;
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
