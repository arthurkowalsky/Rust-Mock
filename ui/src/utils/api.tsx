
import { Endpoint, EndpointResponse, RequestLog, ServerConfig, ProxyConfig } from "@/types";
import { toast } from "sonner";

const getBaseUrl = (): string => {
  if (typeof window !== 'undefined') {
    return window.location.origin;
  }
  return '';
};

const SERVER_CONFIG_KEY = "rustmock_server_config";
const TEST_HISTORY_KEY = "rustmock_test_history";

export const getServerConfig = (): ServerConfig => {
  if (typeof window !== 'undefined') {
    const savedConfig = localStorage.getItem(SERVER_CONFIG_KEY);
    if (savedConfig) {
      try {
        return JSON.parse(savedConfig);
      } catch (e) {
        console.error("Failed to parse server config:", e);
      }
    }
  }
  
  return {
    host: window.location.hostname,
    port: Number(window.location.port) || (window.location.protocol === 'https:' ? 443 : 80)
  };
};

export const saveServerConfig = (host: string, port: number): void => {
  if (typeof window !== 'undefined') {
    const config: ServerConfig = { host, port };
    localStorage.setItem(SERVER_CONFIG_KEY, JSON.stringify(config));
    toast.success("Server configuration saved");
  }
};

export const getTestHistory = (): any[] => {
  if (typeof window !== 'undefined') {
    const savedHistory = localStorage.getItem(TEST_HISTORY_KEY);
    if (savedHistory) {
      try {
        const history = JSON.parse(savedHistory);
        return Array.isArray(history) ? history : [];
      } catch (e) {
        console.error("Failed to parse test history:", e);
      }
    }
  }
  return [];
};

export const saveTestHistory = (
  method: string, 
  url: string, 
  headers: Record<string, string>, 
  body?: any
): void => {
  if (typeof window !== 'undefined') {
    const history = getTestHistory();
    const newEntry = {
      method,
      url,
      headers,
      body,
      timestamp: new Date().toISOString()
    };
    
    history.unshift(newEntry);
    
    const limitedHistory = history.slice(0, 20);
    
    localStorage.setItem(TEST_HISTORY_KEY, JSON.stringify(limitedHistory));
  }
};

export const fetchEndpoints = async (): Promise<Endpoint[]> => {
  try {
    const response = await fetch(`${getBaseUrl()}/__mock/config`);
    if (!response.ok) {
      throw new Error(`Error fetching endpoints: ${response.statusText}`);
    }
    return await response.json();
  } catch (error) {
    console.error("Failed to fetch endpoints:", error);
    toast.error("Failed to fetch endpoints. Check if Rust Mock server is running.");
    return [];
  }
};

export const addEndpoint = async (endpoint: Endpoint): Promise<EndpointResponse> => {
  try {
    const response = await fetch(`${getBaseUrl()}/__mock/endpoints`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(endpoint),
    });
    
    if (!response.ok) {
      throw new Error(`Error adding endpoint: ${response.statusText}`);
    }
    
    const result = await response.json();
    
    if (result.added) {
      toast.success(`Endpoint ${endpoint.method} ${endpoint.path} added successfully`);
    }
    
    return result;
  } catch (error) {
    console.error("Failed to add endpoint:", error);
    toast.error("Failed to add endpoint. Check if Rust Mock server is running.");
    return { error: String(error) };
  }
};

export const removeEndpoint = async (method: string, path: string): Promise<EndpointResponse> => {
  try {
    const response = await fetch(`${getBaseUrl()}/__mock/endpoints`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ method, path }),
    });
    
    if (!response.ok) {
      throw new Error(`Error removing endpoint: ${response.statusText}`);
    }
    
    const result = await response.json();
    
    if (result.removed) {
      toast.success(`Endpoint ${method} ${path} removed successfully`);
    } else {
      toast.error(`Endpoint ${method} ${path} not found`);
    }
    
    return result;
  } catch (error) {
    console.error("Failed to remove endpoint:", error);
    toast.error("Failed to remove endpoint. Check if Rust Mock server is running.");
    return { error: String(error) };
  }
};

export const fetchLogs = async (): Promise<RequestLog[]> => {
  try {
    const response = await fetch(`${getBaseUrl()}/__mock/logs`);
    if (!response.ok) {
      throw new Error(`Error fetching logs: ${response.statusText}`);
    }
    return await response.json();
  } catch (error) {
    console.error("Failed to fetch logs:", error);
    toast.error("Failed to fetch logs. Check if Rust Mock server is running.");
    return [];
  }
};

export const testEndpoint = async (
  method: string,
  url: string,
  headers: Record<string, string> = {},
  body?: any
): Promise<{
  status: number;
  headers: Record<string, string>;
  data: any;
  time: number;
}> => {
  const startTime = performance.now();
  
  try {
    const options: RequestInit = {
      method,
      headers: {
        ...headers,
        "Content-Type": "application/json",
      },
    };

    if (body && method !== "GET") {
      options.body = typeof body === "string" ? body : JSON.stringify(body);
    }

    const fullUrl = url.startsWith("http") ? url : `${getBaseUrl()}${url}`;
    const response = await fetch(fullUrl, options);
    
    const endTime = performance.now();
    const time = Math.round(endTime - startTime);
    
    const responseHeaders: Record<string, string> = {};
    response.headers.forEach((value, key) => {
      responseHeaders[key] = value;
    });
    
    let data;
    const contentType = response.headers.get("content-type");

    if (response.status === 204 || response.status === 304) {
      data = null;
    } else if (contentType?.includes("application/json")) {
      const text = await response.text();
      try {
        data = text.trim() ? JSON.parse(text) : null;
      } catch (e) {
        data = text;
      }
    } else {
      data = await response.text();
    }
    
    return {
      status: response.status,
      headers: responseHeaders,
      data,
      time,
    };
  } catch (error) {
    console.error("Error testing endpoint:", error);
    const endTime = performance.now();
    return {
      status: 0,
      headers: {},
      data: { error: String(error) },
      time: Math.round(endTime - startTime),
    };
  }
};

export const exportEndpoints = (endpoints: Endpoint[]): void => {
  const dataStr = JSON.stringify(endpoints, null, 2);
  const dataUri = `data:application/json;charset=utf-8,${encodeURIComponent(dataStr)}`;

  const exportFileDefaultName = `rustmock-endpoints-${new Date().toISOString().slice(0, 10)}.json`;

  const linkElement = document.createElement("a");
  linkElement.setAttribute("href", dataUri);
  linkElement.setAttribute("download", exportFileDefaultName);
  linkElement.click();
};

export const clearLogs = async (): Promise<void> => {
  try {
    const response = await fetch(`${getBaseUrl()}/__mock/logs`, {
      method: "DELETE",
    });

    if (!response.ok) {
      throw new Error(`Error clearing logs: ${response.statusText}`);
    }

    toast.success("Logs cleared successfully");
  } catch (error) {
    console.error("Failed to clear logs:", error);
    toast.error("Failed to clear logs. Check if Rust Mock server is running.");
  }
};

export const importOpenAPI = async (openApiSpec: any): Promise<EndpointResponse> => {
  try {
    const response = await fetch(`${getBaseUrl()}/__mock/import`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ openapi_spec: openApiSpec }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || `Error importing OpenAPI: ${response.statusText}`);
    }

    const result = await response.json();

    if (result.imported) {
      toast.success(`Successfully imported ${result.count} endpoint(s) from OpenAPI specification`);
    }

    return result;
  } catch (error) {
    console.error("Failed to import OpenAPI:", error);
    toast.error(`Failed to import OpenAPI: ${error instanceof Error ? error.message : String(error)}`);
    return { error: String(error) };
  }
};

export const exportOpenAPI = async (): Promise<void> => {
  try {
    const response = await fetch(`${getBaseUrl()}/__mock/export`);

    if (!response.ok) {
      throw new Error(`Error exporting OpenAPI: ${response.statusText}`);
    }

    const openApiSpec = await response.json();

    const dataStr = JSON.stringify(openApiSpec, null, 2);
    const dataUri = `data:application/json;charset=utf-8,${encodeURIComponent(dataStr)}`;

    const exportFileDefaultName = `openapi-spec-${new Date().toISOString().slice(0, 10)}.json`;

    const linkElement = document.createElement("a");
    linkElement.setAttribute("href", dataUri);
    linkElement.setAttribute("download", exportFileDefaultName);
    linkElement.click();

    toast.success("OpenAPI specification exported successfully");
  } catch (error) {
    console.error("Failed to export OpenAPI:", error);
    toast.error("Failed to export OpenAPI. Check if Rust Mock server is running.");
  }
};

export const getProxyConfig = async (): Promise<ProxyConfig | null> => {
  try {
    const response = await fetch(`${getBaseUrl()}/__mock/proxy`);
    if (!response.ok) {
      throw new Error(`Error fetching proxy config: ${response.statusText}`);
    }
    return await response.json();
  } catch (error) {
    console.error("Failed to fetch proxy config:", error);
    toast.error("Failed to fetch proxy configuration");
    return null;
  }
};

export const setProxyConfig = async (url: string): Promise<ProxyConfig | null> => {
  try {
    const response = await fetch(`${getBaseUrl()}/__mock/proxy`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ url }),
    });

    if (!response.ok) {
      throw new Error(`Error setting proxy config: ${response.statusText}`);
    }

    const result = await response.json();

    if (result.enabled) {
      toast.success(`Default proxy URL set to: ${result.proxy_url}`);
    } else {
      toast.success("Default proxy disabled");
    }

    return result;
  } catch (error) {
    console.error("Failed to set proxy config:", error);
    toast.error("Failed to set proxy configuration");
    return null;
  }
};

export const deleteProxyConfig = async (): Promise<void> => {
  try {
    const response = await fetch(`${getBaseUrl()}/__mock/proxy`, {
      method: "DELETE",
    });

    if (!response.ok) {
      throw new Error(`Error deleting proxy config: ${response.statusText}`);
    }

    toast.success("Default proxy removed");
  } catch (error) {
    console.error("Failed to delete proxy config:", error);
    toast.error("Failed to delete proxy configuration");
  }
};
