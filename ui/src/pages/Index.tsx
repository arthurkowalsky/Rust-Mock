import React, { useState, useEffect } from "react";
import Layout from "@/components/Layout";
import EndpointTable from "@/components/EndpointTable";
import LogTable from "@/components/LogTable";
import AddEndpointForm from "@/components/AddEndpointForm";
import TestEndpoint from "@/components/TestEndpoint";
import { Endpoint } from "@/types";
import { fetchEndpoints, fetchLogs, exportEndpoints } from "@/utils/api";
import { Button } from "@/components/ui/button";
import { Download } from "lucide-react";
import { toast } from "sonner";

const Index = () => {
  const [activeTab, setActiveTab] = useState("endpoints");
  const [endpoints, setEndpoints] = useState<Endpoint[]>([]);
  const [logs, setLogs] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [testEndpoint, setTestEndpoint] = useState<Endpoint | undefined>(undefined);
  
  const loadEndpoints = async () => {
    try {
      setLoading(true);
      const data = await fetchEndpoints();
      setEndpoints(data);
    } catch (error) {
      console.error("Failed to load endpoints:", error);
    } finally {
      setLoading(false);
    }
  };
  
  const loadLogs = async () => {
    try {
      setLoading(true);
      const data = await fetchLogs();
      setLogs(data);
    } catch (error) {
      console.error("Failed to load logs:", error);
    } finally {
      setLoading(false);
    }
  };
  
  useEffect(() => {
    if (activeTab === "endpoints") {
      loadEndpoints();
    } else if (activeTab === "logs") {
      loadLogs();
    }
  }, [activeTab]);
  
  useEffect(() => {
    if (activeTab === "test-endpoint" && testEndpoint) {
      setTimeout(() => setTestEndpoint(undefined), 100);
    }
  }, [activeTab]);
  
  const handleTestEndpoint = (endpoint: Endpoint) => {
    setTestEndpoint(endpoint);
    setActiveTab("test-endpoint");
  };
  
  const handleExportEndpoints = () => {
    if (endpoints.length === 0) {
      toast.error("No endpoints to export");
      return;
    }
    exportEndpoints(endpoints);
    toast.success("Endpoints exported successfully");
  };
  
  const handleImportEndpoints = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    
    input.onchange = (e: any) => {
      const file = e.target.files[0];
      if (!file) return;
      
      const reader = new FileReader();
      reader.onload = async (event) => {
        try {
          const importedEndpoints = JSON.parse(event.target?.result as string);
          
          if (!Array.isArray(importedEndpoints)) {
            toast.error("Invalid import file format");
            return;
          }
          
          toast.info("Import functionality will be added in a future update");
        } catch (error) {
          toast.error("Failed to parse import file");
          console.error(error);
        }
      };
      
      reader.readAsText(file);
    };
    
    input.click();
  };
  
  const renderTabContent = () => {
    switch (activeTab) {
      case "endpoints":
        return (
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <h2 className="text-2xl font-bold">Endpoints</h2>
              <div className="flex space-x-2">
                <Button variant="outline" size="sm" onClick={handleExportEndpoints}>
                  <Download className="h-4 w-4 mr-2" />
                  Export
                </Button>
              </div>
            </div>
            <EndpointTable
              endpoints={endpoints}
              onRemove={loadEndpoints}
              onTest={handleTestEndpoint}
            />
          </div>
        );
      case "logs":
        return (
          <div className="space-y-4">
            <h2 className="text-2xl font-bold">Request Logs</h2>
            <LogTable logs={logs} onRefresh={loadLogs} />
          </div>
        );
      case "add-endpoint":
        return (
          <div className="space-y-4">
            <h2 className="text-2xl font-bold">Add Endpoint</h2>
            <AddEndpointForm
              onSuccess={() => {
                loadEndpoints();
                setActiveTab("endpoints");
                toast.success("Endpoint added successfully");
              }}
              onTest={handleTestEndpoint}
            />
          </div>
        );
      case "test-endpoint":
        return (
          <div className="space-y-4">
            <h2 className="text-2xl font-bold">Test Endpoint</h2>
            <TestEndpoint initialEndpoint={testEndpoint} />
          </div>
        );
      default:
        return null;
    }
  };
  
  return (
    <Layout activeTab={activeTab} setActiveTab={setActiveTab}>
      <div className="container mx-auto animate-fade-in">
        {renderTabContent()}
      </div>
    </Layout>
  );
};

export default Index;
