
import { Endpoint, EndpointResponse, RequestLog, ServerConfig } from "@/types";
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

    // If URL is relative, prepend base URL
    const fullUrl = url.startsWith("http") ? url : `${getBaseUrl()}${url}`;
    const response = await fetch(fullUrl, options);
    
    const endTime = performance.now();
    const time = Math.round(endTime - startTime);
    
    // Get response headers
    const responseHeaders: Record<string, string> = {};
    response.headers.forEach((value, key) => {
      responseHeaders[key] = value;
    });
    
    // Get response data
    let data;
    const contentType = response.headers.get("content-type");
    if (contentType?.includes("application/json")) {
      data = await response.json();
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
