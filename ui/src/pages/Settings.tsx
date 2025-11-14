import React, { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import Layout from "@/components/Layout";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { getProxyConfig, setProxyConfig, deleteProxyConfig } from "@/utils/api";
import { ProxyConfig } from "@/types";
import { toast } from "sonner";
import { Settings as SettingsIcon, Globe, Power, Trash2, RefreshCw, ArrowLeft } from "lucide-react";

const Settings = () => {
  const [proxyConfig, setProxyConfigState] = useState<ProxyConfig | null>(null);
  const [proxyUrl, setProxyUrl] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(true);
  const [saving, setSaving] = useState<boolean>(false);

  const loadProxyConfig = async () => {
    try {
      setLoading(true);
      const config = await getProxyConfig();
      setProxyConfigState(config);
      if (config?.proxy_url) {
        setProxyUrl(config.proxy_url);
      }
    } catch (error) {
      console.error("Failed to load proxy config:", error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadProxyConfig();
  }, []);

  const handleSave = async () => {
    if (!proxyUrl.trim()) {
      toast.error("Please enter a valid proxy URL");
      return;
    }

    // Basic URL validation
    try {
      new URL(proxyUrl.trim());
    } catch {
      toast.error("Invalid URL format. Please enter a valid URL (e.g., https://api.example.com)");
      return;
    }

    try {
      setSaving(true);
      const result = await setProxyConfig(proxyUrl.trim());
      if (result) {
        setProxyConfigState(result);
      }
    } catch (error) {
      console.error("Failed to save proxy config:", error);
    } finally {
      setSaving(false);
    }
  };

  const handleClear = async () => {
    try {
      setSaving(true);
      await deleteProxyConfig();
      setProxyConfigState({ proxy_url: null, enabled: false });
      setProxyUrl("");
    } catch (error) {
      console.error("Failed to clear proxy config:", error);
    } finally {
      setSaving(false);
    }
  };

  const handleTest = async () => {
    if (!proxyUrl.trim()) {
      toast.error("Please enter a proxy URL first");
      return;
    }

    try {
      const testUrl = new URL(proxyUrl.trim());
      toast.info(`Testing connection to ${testUrl.host}...`);
      
      // Simple connectivity test
      const response = await fetch(proxyUrl.trim(), { method: 'HEAD', mode: 'no-cors' });
      toast.success("Connection test successful!");
    } catch (error) {
      toast.warning("Could not verify connection. The URL may still work for proxying.");
    }
  };

  return (
    <Layout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <SettingsIcon className="w-8 h-8" />
            <div>
              <h1 className="text-3xl font-bold">Settings</h1>
              <p className="text-muted-foreground">Configure RustMock server settings</p>
            </div>
          </div>
          <Link to="/">
            <Button variant="outline">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Dashboard
            </Button>
          </Link>
        </div>

        {/* Proxy Configuration Card */}
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Globe className="w-6 h-6" />
                <div>
                  <CardTitle>Smart Proxy Mode</CardTitle>
                  <CardDescription>
                    Configure default proxy URL for unmocked endpoints
                  </CardDescription>
                </div>
              </div>
              {proxyConfig?.enabled ? (
                <Badge variant="default" className="flex items-center gap-1">
                  <Power className="w-3 h-3" />
                  Enabled
                </Badge>
              ) : (
                <Badge variant="secondary" className="flex items-center gap-1">
                  <Power className="w-3 h-3" />
                  Disabled
                </Badge>
              )}
            </div>
          </CardHeader>
          <CardContent className="space-y-6">
            {loading ? (
              <div className="flex items-center justify-center py-8">
                <RefreshCw className="w-6 h-6 animate-spin" />
                <span className="ml-2">Loading configuration...</span>
              </div>
            ) : (
              <>
                {/* Current Status */}
                {proxyConfig?.enabled && proxyConfig.proxy_url && (
                  <Alert>
                    <AlertDescription>
                      <div className="space-y-1">
                        <p className="font-medium">Active Proxy Configuration:</p>
                        <code className="text-sm bg-muted px-2 py-1 rounded">
                          {proxyConfig.proxy_url}
                        </code>
                        <p className="text-xs text-muted-foreground mt-2">
                          All unmocked endpoints will be forwarded to this URL
                        </p>
                      </div>
                    </AlertDescription>
                  </Alert>
                )}

                {/* Configuration Form */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label htmlFor="proxy-url">Default Proxy URL</Label>
                    <Input
                      id="proxy-url"
                      type="url"
                      placeholder="https://api.production.com"
                      value={proxyUrl}
                      onChange={(e) => setProxyUrl(e.target.value)}
                      disabled={saving}
                    />
                    <p className="text-sm text-muted-foreground">
                      Enter the base URL of the API to proxy unmocked requests to
                    </p>
                  </div>

                  {/* Action Buttons */}
                  <div className="flex gap-3">
                    <Button
                      onClick={handleSave}
                      disabled={saving || !proxyUrl.trim()}
                    >
                      {saving ? "Saving..." : "Save Configuration"}
                    </Button>
                    <Button
                      variant="outline"
                      onClick={handleTest}
                      disabled={saving || !proxyUrl.trim()}
                    >
                      Test Connection
                    </Button>
                    {proxyConfig?.enabled && (
                      <Button
                        variant="destructive"
                        onClick={handleClear}
                        disabled={saving}
                      >
                        <Trash2 className="w-4 h-4 mr-2" />
                        Clear Proxy
                      </Button>
                    )}
                  </div>
                </div>

                {/* Information Box */}
                <Alert>
                  <AlertDescription>
                    <div className="space-y-2">
                      <p className="font-semibold">How Smart Proxy Mode Works:</p>
                      <ul className="list-disc list-inside space-y-1 text-sm">
                        <li>Endpoints with mock responses return the configured mock</li>
                        <li>Endpoints with <code>proxy_url</code> forward to that URL</li>
                        <li>All other requests forward to this default proxy URL</li>
                        <li>Headers, query params, and request bodies are preserved</li>
                      </ul>
                      <p className="text-xs text-muted-foreground mt-3">
                        Perfect for testing new endpoints while using production data for stable APIs
                      </p>
                    </div>
                  </AlertDescription>
                </Alert>

                {/* Source Info */}
                <div className="pt-4 border-t">
                  <p className="text-sm text-muted-foreground">
                    <strong>Configuration Source:</strong> 
                    {proxyConfig?.enabled ? (
                      <span className="ml-2">Runtime (can be changed via API or UI)</span>
                    ) : (
                      <span className="ml-2">Not configured (can be set via ENV, CLI, or UI)</span>
                    )}
                  </p>
                  <p className="text-xs text-muted-foreground mt-1">
                    Priority: CLI args → Runtime config → Environment variable
                  </p>
                </div>
              </>
            )}
          </CardContent>
        </Card>

        {/* Additional Settings Placeholder */}
        <Card className="opacity-50">
          <CardHeader>
            <CardTitle>Server Information</CardTitle>
            <CardDescription>View server details and statistics</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">Coming soon...</p>
          </CardContent>
        </Card>
      </div>
    </Layout>
  );
};

export default Settings;
