
import React, { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import { Endpoint, HttpMethod, KeyValuePair } from "@/types";
import { testEndpoint, saveTestHistory, getTestHistory } from "@/utils/api";
import JsonEditor from "./JsonEditor";
import KeyValueEditor from "./KeyValueEditor";
import { toast } from "sonner";
import { Hourglass, ClipboardCopy, Check } from "lucide-react";

interface TestEndpointProps {
  initialEndpoint?: Endpoint;
}

const TestEndpoint: React.FC<TestEndpointProps> = ({ initialEndpoint }) => {
  const [method, setMethod] = useState<HttpMethod>(initialEndpoint?.method || "GET");
  const [url, setUrl] = useState(initialEndpoint?.path || "");
  const [headerPairs, setHeaderPairs] = useState<KeyValuePair[]>([]);
  const [requestBody, setRequestBody] = useState<any>(initialEndpoint?.response || {});
  const [isTesting, setIsTesting] = useState(false);
  const [history, setHistory] = useState<any[]>([]);
  const [copied, setCopied] = useState(false);
  
  // Response state
  const [response, setResponse] = useState<{
    status: number;
    time: number;
    headers: Record<string, string>;
    data: any;
  } | null>(null);

  // Load test history on component mount
  useEffect(() => {
    setHistory(getTestHistory());
  }, []);

  // Set up the initial endpoint if provided
  useEffect(() => {
    if (initialEndpoint) {
      setMethod(initialEndpoint.method);
      setUrl(initialEndpoint.path);
      setRequestBody(initialEndpoint.response || {});
      
      // Convert headers object to key-value pairs
      if (initialEndpoint.headers) {
        const pairs = Object.entries(initialEndpoint.headers).map(([key, value]) => ({
          key,
          value: String(value),
        }));
        setHeaderPairs(pairs);
      }
    }
  }, [initialEndpoint]);

  // Handle headers change
  const handleHeadersChange = (pairs: KeyValuePair[]) => {
    setHeaderPairs(pairs);
  };

  // Convert header pairs to a headers object
  const getHeadersObject = () => {
    const headers: Record<string, string> = {};
    headerPairs.forEach((pair) => {
      if (pair.key && pair.key.trim()) {
        headers[pair.key.trim()] = pair.value;
      }
    });
    return headers;
  };

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!url) {
      toast.error("URL is required");
      return;
    }
    
    try {
      setIsTesting(true);
      setResponse(null);
      
      // Get headers object from key-value pairs
      const headers = getHeadersObject();
      
      // Make the test request
      const result = await testEndpoint(method, url, headers, method !== "GET" ? requestBody : undefined);
      
      // Update the response state
      setResponse(result);
      
      // Save to test history
      saveTestHistory(method, url, headers, requestBody);
      
      // Update history in state
      setHistory(getTestHistory());
    } catch (error) {
      console.error("Error testing endpoint:", error);
      toast.error("Test failed: " + String(error));
    } finally {
      setIsTesting(false);
    }
  };

  // Handle loading from history
  const handleLoadFromHistory = (entry: any) => {
    setMethod(entry.method as HttpMethod);
    setUrl(entry.url);
    setRequestBody(entry.body || {});
    
    // Convert headers object to key-value pairs
    if (entry.headers) {
      const pairs = Object.entries(entry.headers).map(([key, value]) => ({
        key,
        value: String(value),
      }));
      setHeaderPairs(pairs);
    } else {
      setHeaderPairs([]);
    }
  };

  // Copy response as JSON
  const handleCopyResponse = () => {
    if (response) {
      const responseText = JSON.stringify(response.data, null, 2);
      navigator.clipboard.writeText(responseText);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="space-y-6">
      {/* Test Form */}
      <div className="bg-white rounded-lg shadow p-6">
        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="flex flex-col md:flex-row gap-4">
            {/* Method Selector */}
            <div className="w-full md:w-40">
              <Select
                value={method}
                onValueChange={(value) => setMethod(value as HttpMethod)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Method" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="GET">GET</SelectItem>
                  <SelectItem value="POST">POST</SelectItem>
                  <SelectItem value="PUT">PUT</SelectItem>
                  <SelectItem value="PATCH">PATCH</SelectItem>
                  <SelectItem value="DELETE">DELETE</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* URL Input */}
            <div className="flex-1">
              <Input
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="http://127.0.0.1:8080/api/endpoint or /api/endpoint"
              />
            </div>

            {/* Send Button */}
            <div>
              <Button type="submit" disabled={isTesting}>
                {isTesting ? (
                  <>
                    <Hourglass className="h-4 w-4 mr-2 animate-spin" />
                    Testing...
                  </>
                ) : (
                  "Send"
                )}
              </Button>
            </div>
          </div>

          {/* Request Tabs - Headers & Body */}
          <Tabs defaultValue="body">
            <TabsList>
              <TabsTrigger value="body">Body</TabsTrigger>
              <TabsTrigger value="headers">Headers</TabsTrigger>
              <TabsTrigger value="history">History</TabsTrigger>
            </TabsList>

            <TabsContent value="body" className="pt-4">
              <div className="space-y-2">
                <Label>Request Body (JSON)</Label>
                <JsonEditor
                  value={requestBody}
                  onChange={setRequestBody}
                  placeholder='{"key": "value"}'
                  height={method === "GET" ? "h-32" : "h-64"}
                />
                {method === "GET" && (
                  <p className="text-amber-600 text-xs mt-1">
                    Note: GET requests typically don't have a request body. 
                    The body will be ignored for GET requests.
                  </p>
                )}
              </div>
            </TabsContent>

            <TabsContent value="headers" className="pt-4">
              <div className="space-y-2">
                <Label>Request Headers</Label>
                <KeyValueEditor
                  pairs={headerPairs}
                  onChange={handleHeadersChange}
                  placeholder={{ key: "Header", value: "Value" }}
                />
              </div>
            </TabsContent>

            <TabsContent value="history" className="pt-4">
              <div className="space-y-2">
                <Label>Request History</Label>
                {history.length > 0 ? (
                  <div className="space-y-2 max-h-64 overflow-auto">
                    {history.map((entry, index) => (
                      <div
                        key={index}
                        className="p-3 border rounded-md hover:bg-gray-50 cursor-pointer"
                        onClick={() => handleLoadFromHistory(entry)}
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center">
                            <span
                              className={`px-2 py-1 rounded text-xs font-medium method-${entry.method.toLowerCase()}`}
                            >
                              {entry.method}
                            </span>
                            <span className="ml-2 font-mono text-sm truncate">
                              {entry.url}
                            </span>
                          </div>
                          <span className="text-xs text-gray-500">
                            {new Date(entry.timestamp).toLocaleString()}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-gray-500 py-4 text-center">
                    No history available
                  </p>
                )}
              </div>
            </TabsContent>
          </Tabs>
        </form>
      </div>

      {/* Response Section */}
      {response && (
        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-lg font-medium">Response</h2>
            <div className="flex items-center space-x-2">
              <div className="text-sm text-gray-500">
                {response.time}ms
              </div>
              <div>
                <span
                  className={`px-2 py-1 rounded text-xs font-medium ${
                    response.status >= 200 && response.status < 300
                      ? "bg-green-100 text-green-800"
                      : response.status >= 300 && response.status < 400
                      ? "bg-blue-100 text-blue-800"
                      : response.status >= 400 && response.status < 500
                      ? "bg-yellow-100 text-yellow-800"
                      : "bg-red-100 text-red-800"
                  }`}
                >
                  {response.status}
                </span>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleCopyResponse}
                className="text-xs"
              >
                {copied ? (
                  <Check className="h-4 w-4 mr-1" />
                ) : (
                  <ClipboardCopy className="h-4 w-4 mr-1" />
                )}
                {copied ? "Copied" : "Copy"}
              </Button>
            </div>
          </div>

          <Tabs defaultValue="body">
            <TabsList>
              <TabsTrigger value="body">Body</TabsTrigger>
              <TabsTrigger value="headers">Headers</TabsTrigger>
            </TabsList>

            <TabsContent value="body" className="pt-4">
              <JsonEditor
                value={response.data}
                onChange={() => {}}
                height="h-64"
              />
            </TabsContent>

            <TabsContent value="headers" className="pt-4">
              <div className="border rounded-md p-3 font-mono text-sm max-h-64 overflow-auto">
                {Object.entries(response.headers).map(([key, value]) => (
                  <div key={key}>
                    <span className="text-purple-600">{key}</span>:{" "}
                    <span>{value}</span>
                  </div>
                ))}
              </div>
            </TabsContent>
          </Tabs>
        </div>
      )}
    </div>
  );
};

export default TestEndpoint;
