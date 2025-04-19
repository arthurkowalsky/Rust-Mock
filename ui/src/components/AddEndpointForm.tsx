
import React, { useState } from "react";
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
import { addEndpoint, testEndpoint } from "@/utils/api";
import JsonEditor from "./JsonEditor";
import KeyValueEditor from "./KeyValueEditor";
import { toast } from "sonner";

const DEFAULT_ENDPOINT: Endpoint = {
  method: "GET",
  path: "",
  response: {},
  status: 200,
  headers: {},
};

interface AddEndpointFormProps {
  onSuccess: () => void;
  onTest: (endpoint: Endpoint) => void;
}

const AddEndpointForm: React.FC<AddEndpointFormProps> = ({ onSuccess, onTest }) => {
  const [endpoint, setEndpoint] = useState<Endpoint>({ ...DEFAULT_ENDPOINT });
  const [headerPairs, setHeaderPairs] = useState<KeyValuePair[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isValidPath, setIsValidPath] = useState(true);

  // Validate path format
  const validatePath = (path: string): boolean => {
    // Check if path starts with '/'
    return path.startsWith("/");
  };

  // Handle change in form fields
  const handleChange = (
    field: keyof Endpoint,
    value: string | number | object
  ) => {
    const updatedEndpoint = { ...endpoint };
    
    if (field === "path") {
      const pathString = value as string;
      setIsValidPath(validatePath(pathString));
      updatedEndpoint[field] = pathString;
    } else if (field === "method") {
      updatedEndpoint[field] = value as HttpMethod;
    } else if (field === "status") {
      updatedEndpoint[field] = Number(value);
    } else if (field === "response") {
      updatedEndpoint[field] = value;
    }
    
    setEndpoint(updatedEndpoint);
  };

  // Handle headers change
  const handleHeadersChange = (pairs: KeyValuePair[]) => {
    setHeaderPairs(pairs);
    
    // Convert key-value pairs to a headers object
    const headersObj: Record<string, string> = {};
    pairs.forEach((pair) => {
      if (pair.key && pair.key.trim()) {
        headersObj[pair.key.trim()] = pair.value;
      }
    });
    
    setEndpoint((prev) => ({
      ...prev,
      headers: headersObj,
    }));
  };

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!isValidPath) {
      toast.error("Path must start with '/'");
      return;
    }
    
    try {
      setIsSubmitting(true);
      const result = await addEndpoint(endpoint);
      
      if (result.added) {
        onSuccess();
        setEndpoint({ ...DEFAULT_ENDPOINT });
        setHeaderPairs([]);
      }
    } catch (error) {
      console.error("Error adding endpoint:", error);
      toast.error("Failed to add endpoint");
    } finally {
      setIsSubmitting(false);
    }
  };

  // Handle testing the endpoint
  const handleTest = () => {
    if (!isValidPath) {
      toast.error("Path must start with '/'");
      return;
    }
    
    onTest(endpoint);
  };

  return (
    <div className="space-y-6">
      <div className="bg-white rounded-lg shadow p-6">
        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {/* Method Field */}
            <div className="space-y-2">
              <Label htmlFor="method">HTTP Method</Label>
              <Select
                value={endpoint.method}
                onValueChange={(value) => handleChange("method", value)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select method" />
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

            {/* Path Field */}
            <div className="space-y-2">
              <Label htmlFor="path">
                Path
                <span className="text-red-500 ml-1">*</span>
              </Label>
              <Input
                id="path"
                value={endpoint.path}
                onChange={(e) => handleChange("path", e.target.value)}
                placeholder="/api/users"
                className={!isValidPath ? "border-red-500" : ""}
              />
              {!isValidPath && (
                <p className="text-red-500 text-xs mt-1">
                  Path must start with "/"
                </p>
              )}
            </div>

            {/* Status Field */}
            <div className="space-y-2">
              <Label htmlFor="status">Status Code</Label>
              <Input
                id="status"
                type="number"
                value={endpoint.status}
                onChange={(e) => handleChange("status", e.target.value)}
                placeholder="200"
              />
            </div>
          </div>

          {/* Tabs for Response & Headers */}
          <Tabs defaultValue="response">
            <TabsList>
              <TabsTrigger value="response">Response</TabsTrigger>
              <TabsTrigger value="headers">Headers</TabsTrigger>
            </TabsList>
            
            <TabsContent value="response" className="pt-4">
              <div className="space-y-2">
                <Label>Response Body (JSON)</Label>
                <JsonEditor
                  value={endpoint.response}
                  onChange={(value) => handleChange("response", value)}
                  placeholder='{"message": "Success", "data": {...}}'
                />
              </div>
            </TabsContent>
            
            <TabsContent value="headers" className="pt-4">
              <div className="space-y-2">
                <Label>Response Headers</Label>
                <KeyValueEditor
                  pairs={headerPairs}
                  onChange={handleHeadersChange}
                  placeholder={{ key: "Header", value: "Value" }}
                />
              </div>
            </TabsContent>
          </Tabs>

          {/* Action Buttons */}
          <div className="flex justify-end space-x-3">
            <Button
              type="button"
              variant="outline"
              onClick={handleTest}
              disabled={isSubmitting}
            >
              Test
            </Button>
            <Button type="submit" disabled={isSubmitting || !isValidPath}>
              {isSubmitting ? "Adding..." : "Add Endpoint"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default AddEndpointForm;
